// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Native backing for `TextDecoder`'s per-instance streaming state.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use encoding_rs::{CoderResult, Decoder, DecoderResult, Encoding};
use script_engine_api::{CallCx, NativeFn, ScriptEngine};

use crate::HostState;

static NEXT_DECODER_ID: AtomicU64 = AtomicU64::new(1);

/// IDs are process-wide and checked so a stale JS cleanup job can never name a
/// later decoder, including after a runtime snapshot replaces HostState.
fn allocate_decoder_id() -> Option<u64> {
    NEXT_DECODER_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
}

#[derive(Default)]
pub(crate) struct DecoderRegistry {
    entries: HashMap<u64, DecoderEntry>,
}

impl DecoderRegistry {
    fn insert(&mut self, id: u64, entry: DecoderEntry) {
        self.entries.insert(id, entry);
    }

    fn decode(&mut self, id: u64, bytes: &[u8], stream: bool) -> Option<Result<String, ()>> {
        self.entries
            .get_mut(&id)
            .map(|entry| entry.decode(bytes, stream))
    }

    fn release(&mut self, id: u64) {
        self.entries.remove(&id);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

struct DecoderEntry {
    encoding: &'static Encoding,
    fatal: bool,
    ignore_bom: bool,
    decoder: Option<Decoder>,
    pending: Vec<u8>,
}

impl DecoderEntry {
    fn new(encoding: &'static Encoding, fatal: bool, ignore_bom: bool) -> Self {
        Self {
            encoding,
            fatal,
            ignore_bom,
            decoder: None,
            pending: Vec::new(),
        }
    }

    fn decode(&mut self, bytes: &[u8], stream: bool) -> Result<String, ()> {
        // An empty streaming chunk cannot change decoder state when there is no
        // caller-unconsumed input to retry. In particular, do not ask legacy
        // decoders to revisit a lead byte buffered internally, or let an empty
        // chunk disturb initial BOM detection.
        if stream && bytes.is_empty() && self.pending.is_empty() {
            return Ok(String::new());
        }

        let mut input = std::mem::take(&mut self.pending);
        input.extend_from_slice(bytes);
        let fatal = self.fatal;
        if self.decoder.is_none() {
            self.decoder = Some(if self.ignore_bom {
                self.encoding.new_decoder_without_bom_handling()
            } else {
                self.encoding.new_decoder_with_bom_removal()
            });
        }
        let DecoderEntry {
            decoder: decoder_slot,
            pending,
            ..
        } = self;
        let active_decoder = decoder_slot.as_mut().expect("decoder initialized above");
        let mut output = String::new();
        let mut offset = 0;
        loop {
            // Reserve enough for the worst case and append directly. On
            // OutputFull the decoder has consumed only the reported prefix.
            let reserve = active_decoder
                .max_utf8_buffer_length_without_replacement(input.len() - offset)
                .unwrap_or((input.len() - offset).saturating_mul(4).saturating_add(16));
            output.reserve(reserve.max(16));
            enum Step {
                InputEmpty,
                OutputFull,
                Malformed,
            }
            let (step, read) = if fatal {
                let (result, read) = active_decoder.decode_to_string_without_replacement(
                    &input[offset..],
                    &mut output,
                    !stream,
                );
                let step = match result {
                    DecoderResult::InputEmpty => Step::InputEmpty,
                    DecoderResult::OutputFull => Step::OutputFull,
                    DecoderResult::Malformed(_, _) => Step::Malformed,
                };
                (step, read)
            } else {
                let (result, read, _had_errors) =
                    active_decoder.decode_to_string(&input[offset..], &mut output, !stream);
                let step = match result {
                    CoderResult::InputEmpty => Step::InputEmpty,
                    CoderResult::OutputFull => Step::OutputFull,
                };
                (step, read)
            };
            offset += read;
            match step {
                Step::InputEmpty => break,
                Step::OutputFull => continue,
                Step::Malformed => {
                    // `read` is the only caller-input offset. The malformed
                    // tuple describes error lengths, while the decoder keeps
                    // any consumed replay bytes internally.
                    if stream {
                        pending.extend_from_slice(&input[offset..]);
                    } else {
                        *decoder_slot = None;
                        pending.clear();
                    }
                    return Err(());
                },
            }
        }
        if !stream {
            *decoder_slot = None;
            pending.clear();
        }
        Ok(output)
    }
}

fn arg_string<E: ScriptEngine>(cx: &mut E::CallCx<'_>, index: usize) -> Result<String, E::Error> {
    let arg = cx.arg(index);
    cx.value_to_string(&arg)
}

struct TextDecoderCreate;
impl<E: ScriptEngine> NativeFn<E> for TextDecoderCreate {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let label = arg_string::<E>(cx, 0)?;
        let fatal = arg_string::<E>(cx, 1)? == "1";
        let ignore_bom = arg_string::<E>(cx, 2)? == "1";
        let encoding = Encoding::for_label(label.as_bytes())
            .filter(|encoding| *encoding != encoding_rs::REPLACEMENT);
        let Some(encoding) = encoding else {
            return cx.make_string("!L");
        };
        let Some(id) = allocate_decoder_id() else {
            return cx.make_string("!H");
        };
        let state = cx.host_data().and_then(|data| {
            data.downcast_ref::<RefCell<HostState>>().map(|host| {
                host.borrow_mut()
                    .text_decoders
                    .insert(id, DecoderEntry::new(encoding, fatal, ignore_bom));
            })
        });
        if state.is_none() {
            return cx.make_string("!H");
        }
        cx.make_string(&format!("+{id}:{}", encoding.name().to_ascii_lowercase()))
    }
}

struct TextDecoderDecode;
impl<E: ScriptEngine> NativeFn<E> for TextDecoderDecode {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = arg_string::<E>(cx, 0)?.parse::<u64>().ok();
        let binary = arg_string::<E>(cx, 1)?;
        let stream = arg_string::<E>(cx, 2)? == "1";
        let bytes: Vec<u8> = binary.chars().map(|ch| ch as u32 as u8).collect();
        let result = cx.host_data().and_then(|data| {
            let host = data.downcast_ref::<RefCell<HostState>>()?;
            let mut host = host.borrow_mut();
            host.text_decoders.decode(id?, &bytes, stream)
        });
        let response = match result {
            Some(Ok(text)) => {
                let mut response = String::with_capacity(text.len().saturating_add(1));
                response.push('S');
                response.push_str(&text);
                response
            },
            Some(Err(())) => String::from("F"),
            None => String::from("H"),
        };
        cx.make_string(&response)
    }
}

struct TextDecoderRelease;
impl<E: ScriptEngine> NativeFn<E> for TextDecoderRelease {
    fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
        let id = arg_string::<E>(cx, 0)?.parse::<u64>().ok();
        if let (Some(id), Some(data)) = (id, cx.host_data()) {
            if let Some(host) = data.downcast_ref::<RefCell<HostState>>() {
                host.borrow_mut().text_decoders.release(id);
            }
        }
        Ok(cx.undefined())
    }
}

pub(crate) fn install_text_encoding_surface<E: ScriptEngine>(
    engine: &mut crate::Surface<'_, '_, E>,
) -> Result<(), crate::SurfaceError<E::Error>> {
    engine.set_function::<TextDecoderCreate>("__text_decoder_create", 3)?;
    engine.set_function::<TextDecoderDecode>("__text_decoder_decode", 3)?;
    engine.set_function::<TextDecoderRelease>("__text_decoder_release", 1)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime_decoder_lifecycle<E: ScriptEngine>() {
        let mut runtime = crate::Runtime::<E>::new().expect("runtime");
        runtime
            .eval(
                "globalThis.decoder=new TextDecoder('utf-8',{fatal:true}); \
                 try { decoder.decode(new Uint8Array([255,65]),{stream:true}); } catch (_) {}",
            )
            .expect("create and stream decoder");
        {
            let host = runtime.host().borrow();
            assert_eq!(host.text_decoders.len(), 1);
            let entry = host.text_decoders.entries.values().next().unwrap();
            assert!(
                entry.decoder.is_some(),
                "streaming decode owns active state"
            );
            assert_eq!(entry.pending, b"A", "unconsumed suffix remains native-side");
        }

        let flushed = runtime.eval("decoder.decode()").expect("flush decoder");
        assert_eq!(runtime.value_to_string(&flushed).unwrap(), "A");
        {
            let host = runtime.host().borrow();
            assert_eq!(
                host.text_decoders.len(),
                1,
                "wrapper metadata remains reusable"
            );
            let entry = host.text_decoders.entries.values().next().unwrap();
            assert!(entry.decoder.is_none(), "flush drops the stream decoder");
            assert!(entry.pending.is_empty(), "flush clears pending input");
        }

        runtime.eval("decoder=null").expect("drop JS wrapper");
        for _ in 0..6 {
            runtime.collect_garbage();
            runtime.run_event_loop(0).expect("pump cleanup jobs");
            if runtime.host().borrow().text_decoders.len() == 0 {
                break;
            }
        }
        assert_eq!(
            runtime.host().borrow().text_decoders.len(),
            0,
            "collection plus normal job pumping releases native metadata"
        );
    }

    fn runtime_drop_clears_native_decoder_state<E: ScriptEngine>() {
        let mut runtime = crate::Runtime::<E>::new().expect("runtime");
        runtime
            .eval("globalThis.decoder=new TextDecoder();")
            .expect("create decoder");
        let retained_host = runtime.host().clone();
        assert_eq!(retained_host.borrow().text_decoders.len(), 1);
        drop(runtime);
        assert!(
            retained_host.borrow().text_decoders.entries.is_empty(),
            "Runtime teardown clears native state even while HostState is retained"
        );
    }

    #[test]
    fn runtime_decoder_lifecycle_on_boa() {
        runtime_decoder_lifecycle::<script_engine_boa::BoaEngine>();
        runtime_drop_clears_native_decoder_state::<script_engine_boa::BoaEngine>();
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn runtime_decoder_lifecycle_on_vano() {
        runtime_decoder_lifecycle::<script_engine_nova::NovaEngine>();
        runtime_drop_clears_native_decoder_state::<script_engine_nova::NovaEngine>();
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn nova_snapshot_stale_decoder_handle_cannot_alias_fresh_state() {
        let mut template =
            crate::Runtime::<script_engine_nova::NovaEngine>::new().expect("runtime");
        template
            .eval("globalThis.staleDecoder=new TextDecoder();")
            .expect("create source decoder");
        let stale_id = *template
            .host()
            .borrow()
            .text_decoders
            .entries
            .keys()
            .next()
            .expect("native entry exists");

        let mut clone = template.snapshot_clone().expect("snapshot clone");
        assert!(clone.host().borrow().text_decoders.entries.is_empty());
        let stale_result = clone
            .eval(
                "(function(){try{staleDecoder.decode();return 'unexpected';}catch(e){return e.name;}})()",
            )
            .expect("stale handle call");
        assert_eq!(clone.value_to_string(&stale_result).unwrap(), "TypeError");
        clone
            .eval("staleDecoder=null;globalThis.freshDecoder=new TextDecoder();")
            .expect("create fresh cloned-runtime decoder");
        let fresh_id = *clone
            .host()
            .borrow()
            .text_decoders
            .entries
            .keys()
            .next()
            .expect("fresh native entry exists");
        assert_ne!(stale_id, fresh_id);

        for _ in 0..6 {
            clone.collect_garbage();
            clone.run_event_loop(0).expect("pump stale cleanup");
            if clone.host().borrow().text_decoders.len() == 1 {
                let value = clone
                    .eval("freshDecoder.decode(new Uint8Array([65]))")
                    .expect("fresh decoder remains live");
                assert_eq!(clone.value_to_string(&value).unwrap(), "A");
                return;
            }
        }
        assert_eq!(clone.host().borrow().text_decoders.len(), 1);
    }

    #[test]
    fn utf8_stream_state_survives_multibyte_splits_and_flush_resets() {
        let mut entry = DecoderEntry::new(encoding_rs::UTF_8, false, false);
        assert_eq!(entry.decode(&[0xE2], true).unwrap(), "");
        assert_eq!(entry.decode(&[0x82], true).unwrap(), "");
        assert_eq!(entry.decode(&[0xAC], true).unwrap(), "€");
        assert!(entry.decoder.is_some());
        assert_eq!(entry.decode(&[], false).unwrap(), "");
        assert!(entry.decoder.is_none());
        assert!(entry.pending.is_empty());
        assert_eq!(entry.decode(b"plain", true).unwrap(), "plain");
    }

    #[test]
    fn bom_option_selects_removal_or_literal_output() {
        let mut remove = DecoderEntry::new(encoding_rs::UTF_8, false, false);
        assert_eq!(remove.decode(b"\xEF\xBB\xBFtext", false).unwrap(), "text");
        let mut keep = DecoderEntry::new(encoding_rs::UTF_8, false, true);
        assert_eq!(
            keep.decode(b"\xEF\xBB\xBFtext", false).unwrap(),
            "\u{FEFF}text"
        );
    }

    #[test]
    fn empty_stream_chunks_preserve_legacy_pending_leads() {
        let cases = [
            (encoding_rs::SHIFT_JIS, 0x81, 0x87, "∞"),
            (encoding_rs::EUC_JP, 0xA4, 0xA2, "あ"),
            (encoding_rs::BIG5, 0xFE, 0x40, "鑂"),
            (encoding_rs::EUC_KR, 0x81, 0x41, "갂"),
        ];

        for (encoding, lead, trail, expected) in cases {
            let mut entry = DecoderEntry::new(encoding, false, false);
            assert_eq!(entry.decode(&[lead], true).unwrap(), "");
            assert!(entry.decoder.is_some(), "lead byte initializes decoder");
            assert!(entry.pending.is_empty());

            assert_eq!(entry.decode(&[], true).unwrap(), "");
            assert!(entry.decoder.is_some(), "empty chunk preserves decoder");
            assert!(entry.pending.is_empty());
            assert_eq!(entry.decode(&[trail], true).unwrap(), expected);
            assert_eq!(entry.decode(&[], false).unwrap(), "");
            assert!(entry.decoder.is_none(), "final flush releases decoder");
        }
    }

    #[test]
    fn empty_stream_chunk_preserves_initial_bom_state_and_final_flushes() {
        let mut entry = DecoderEntry::new(encoding_rs::UTF_8, false, false);
        assert_eq!(entry.decode(&[], true).unwrap(), "");
        assert!(entry.decoder.is_none(), "initial empty chunk stays lazy");

        assert_eq!(entry.decode(&[0xEF], true).unwrap(), "");
        assert!(entry.decoder.is_some());
        assert_eq!(entry.decode(&[], true).unwrap(), "");
        assert!(entry.decoder.is_some(), "empty chunk preserves BOM prefix");
        assert_eq!(entry.decode(&[0xBB, 0xBF, b'x'], true).unwrap(), "x");
        assert_eq!(entry.decode(&[], false).unwrap(), "");
        assert!(entry.decoder.is_none(), "stream=false empty chunk flushes");

        let mut incomplete = DecoderEntry::new(encoding_rs::UTF_8, false, false);
        assert_eq!(incomplete.decode(&[0xE2], true).unwrap(), "");
        assert_eq!(incomplete.decode(&[], false).unwrap(), "\u{FFFD}");
        assert!(incomplete.decoder.is_none());
    }

    #[test]
    fn fatal_stream_error_retains_the_valid_suffix_for_later_input() {
        let mut entry = DecoderEntry::new(encoding_rs::UTF_8, true, false);
        assert!(entry.decode(&[0xFF, b'A'], true).is_err());
        assert_eq!(entry.pending, b"A", "unconsumed suffix remains queued");
        assert_eq!(entry.decode(&[], true).unwrap(), "A");
        assert!(entry.decoder.is_some());
        assert_eq!(entry.decode(&[], false).unwrap(), "");
        assert!(entry.decoder.is_none());
    }

    #[test]
    fn fatal_iso_2022_jp_error_keeps_decoder_replay_state() {
        let mut entry = DecoderEntry::new(encoding_rs::ISO_2022_JP, true, false);
        assert!(entry.decode(&[0x1B, b'(', b'A', b'B'], true).is_err());
        assert_eq!(entry.decode(&[], true).unwrap(), "(AB");
    }

    #[test]
    fn registry_release_is_idempotent_and_ids_do_not_recycle() {
        let first = allocate_decoder_id().unwrap();
        let mut registry = DecoderRegistry::default();
        registry.insert(first, DecoderEntry::new(encoding_rs::UTF_8, false, false));
        registry.release(first);
        registry.release(first);
        assert!(registry.decode(first, b"old", false).is_none());

        let second = allocate_decoder_id().unwrap();
        assert_ne!(first, second);
        registry.insert(second, DecoderEntry::new(encoding_rs::UTF_8, false, false));
        registry.release(first);
        assert_eq!(
            registry.decode(second, b"new", false).unwrap().unwrap(),
            "new"
        );
    }
}
