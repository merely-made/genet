// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use script_engine_api::{CallCx, NativeFn, RealmId, ScriptEngine};

fn relocation_contract<E: ScriptEngine>() {
    struct Cache;
    impl<E: ScriptEngine> NativeFn<E> for Cache {
        fn call(cx: &mut E::CallCx<'_>) -> Result<E::Value, E::Error> {
            let value = cx.arg(0);
            let mode = cx.value_to_string(&value)?;
            let value = cx.arg(1);
            let source: RealmId = cx.value_to_string(&value)?.parse().unwrap();
            let value = cx.arg(2);
            let raw: u64 = cx.value_to_string(&value)?.parse().unwrap();
            let current = cx.current_realm();
            let result = match mode.as_str() {
                "move" => cx.relocate_reflectors(source, raw as RealmId, &[7]),
                "batch" => cx.relocate_reflectors(source, raw as RealmId, &[7, 8]),
                "root" => cx.root_reflector_in_realm(source, raw),
                "release" => cx.unroot_reflector_in_realm(source, raw),
                _ => {
                    return cx
                        .reflector_for_in_realm(source, raw)
                        .map_err(|e| cx.error(&e.to_string()));
                },
            };
            assert_eq!(
                cx.current_realm(),
                current,
                "cache custody changed callback realm"
            );
            cx.make_string(&match result {
                Ok(()) => "ok".into(),
                Err(e) => e.to_string(),
            })
        }
    }
    let mut engine = E::new().unwrap();
    let source = engine.create_realm().unwrap();
    let destination = engine.create_realm().unwrap();
    engine.set_function::<Cache>("cache", 3).unwrap();
    let mut check = |script: String| {
        let value = engine.eval(&script).unwrap();
        assert_eq!(engine.value_to_string(&value).unwrap(), "true", "{script}");
    };
    check(format!(
        "var original=cache('get',{source},7), second=cache('get',{source},8), collision=cache('get',{destination},8); var proto=Object.getPrototypeOf(original); original.marker=23; cache('root',{source},7)==='ok' && second!==collision"
    ));
    check(format!(
        "cache('batch',{source},{destination})!=='ok' && cache('get',{source},7)===original && cache('get',{source},8)===second && cache('get',{destination},8)===collision"
    ));
    drop(check);
    assert_eq!(engine.minted_reflectors_in_realm(source).unwrap().len(), 2);
    assert_eq!(
        engine.minted_reflectors_in_realm(destination).unwrap(),
        vec![8]
    );
    assert_eq!(engine.rooted_reflector_count_in_realm(source).unwrap(), 1);
    let value = engine.eval(&format!("cache('move',{source},{destination})==='ok' && cache('get',{destination},7)===original && Object.getPrototypeOf(original)===proto")).unwrap();
    assert_eq!(engine.value_to_string(&value).unwrap(), "true");
    drop(value);
    assert_eq!(engine.minted_reflectors_in_realm(source).unwrap(), vec![8]);
    assert_eq!(engine.rooted_reflector_count_in_realm(source).unwrap(), 0);
    assert_eq!(
        engine.rooted_reflector_count_in_realm(destination).unwrap(),
        1
    );
    engine.discard_realm(source).unwrap();
    engine.eval("original=second=collision=proto=null").unwrap();
    for _ in 0..3 {
        engine.force_gc();
    }
    let value = engine
        .eval(&format!("cache('get',{destination},7).marker===23"))
        .unwrap();
    assert_eq!(engine.value_to_string(&value).unwrap(), "true");
    drop(value);
    engine
        .eval(&format!("cache('release',{destination},7)"))
        .unwrap();
    for _ in 0..3 {
        engine.force_gc();
    }
    let dead = engine.drain_dead_reflectors_in_realm(destination).unwrap();
    assert!(dead.contains(&7), "transferred root was not released");
    assert!(
        engine
            .minted_reflectors_in_realm(destination)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn boa() {
    relocation_contract::<script_engine_boa::BoaEngine>();
}
#[cfg(target_pointer_width = "64")]
#[test]
fn nova() {
    relocation_contract::<script_engine_nova::NovaEngine>();
}
