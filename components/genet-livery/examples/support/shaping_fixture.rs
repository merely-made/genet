// Shared benchmark fixture generator.
pub fn html(case: &str, paragraphs: usize, para: &str) -> String {
    let count = if case == "single-long" {
        1
    } else if case == "long-short" {
        9
    } else {
        paragraphs
    };
    let mut html = String::from("<!doctype html><html><body>");
    for i in 0..count {
        let content = if case == "single-long" || (case == "long-short" && i == 0) {
            para.repeat(paragraphs)
        } else {
            para.to_owned()
        };
        html.push_str(&format!(
            "<p {}>{content} unique-{i}</p>",
            if i == 0 { "id='edit'" } else { "" }
        ));
    }
    html.push_str("</body></html>");
    html
}

pub fn paragraph(case: &str) -> &'static str {
    match case {
        "small" => "A small line.",
        "mixed" => "English العربية e\u{301} ligature office, punctuation and tabs.",
        "single-long" | "long-short" => {
            "A long paragraph tests wrapping and shaping across many words. "
        },
        "atomic" => {
            "Text with <span style='float:left;width:12px;height:12px'></span><span style='display:inline-block;width:24px;height:12px'>box</span> inline."
        },
        "churn" => "Editing one paragraph among many retained paragraphs.",
        _ => {
            "A paragraph contains several words, punctuation, and enough text to wrap across lines."
        },
    }
}
