use crate::clean::clean_subtitle;
use crate::srt::{SubBlock, Subtitle, TimePoint};

fn map(s: &str) -> Option<String> {
    let mut sub = Subtitle {
        blocks: vec![SubBlock {
            start: TimePoint { ms: 0 },
            end: TimePoint { ms: 0 },
            text: s.to_string(),
        }],
    };

    clean_subtitle(&mut sub);

    match sub.blocks.len() {
        0 => None,
        1 => Some(sub.blocks.pop().unwrap().text),
        _ => panic!("Created an extra block, unexpected"),
    }
}

fn assert_unchanged(s: &str) {
    assert_eq!(Some(s), map(s).as_deref())
}

#[test]
fn time() {
    assert_unchanged("We'll meet at 10:00");
}

#[test]
fn speaker() {
    assert_eq!(Some("Nice"), map("DAVE 5: Nice").as_deref())
}

#[test]
fn speaker_html() {
    assert_eq!(None, map("<color>MAN</color>:").as_deref())
}

#[test]
fn speaker_wonky_digit() {
    assert_eq!(Some("Message"), map("-CH1lD: Message").as_deref())
}
