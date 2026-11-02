use codex::{
    EventError, ItemDelta, ItemDeltaPayload, ItemEnvelope, ItemFailure, ItemPayload, ItemSnapshot,
    ItemStatus, TextContent, TextDelta, ThreadEvent, ThreadStarted, TurnCompleted, TurnFailed,
    TurnStarted,
};
use std::collections::BTreeMap;

fn stored_thread_id(event: &ThreadEvent) -> &String {
    match event {
        ThreadEvent::ThreadStarted(inner) => &inner.thread_id,
        ThreadEvent::TurnStarted(inner) => &inner.thread_id,
        ThreadEvent::TurnCompleted(inner) => &inner.thread_id,
        ThreadEvent::TurnFailed(inner) => &inner.thread_id,
        ThreadEvent::ItemStarted(inner) => &inner.thread_id,
        ThreadEvent::ItemDelta(inner) => &inner.thread_id,
        ThreadEvent::ItemCompleted(inner) => &inner.thread_id,
        ThreadEvent::ItemFailed(inner) => &inner.thread_id,
        ThreadEvent::Error(_) => panic!("ThreadEvent::Error has no thread id"),
    }
}

fn assert_thread_id_borrowed(event: &ThreadEvent, expected: &str) {
    let id = event
        .thread_id()
        .expect("expected thread_id() to return Some");
    assert_eq!(id, expected);

    let stored = stored_thread_id(event);
    assert_eq!(id.as_ptr(), stored.as_ptr());
    assert_eq!(id.len(), stored.len());
}

#[test]
fn thread_id_accessor_is_pinned_and_borrowed() {
    let expected = "thread-123";

    let events = vec![
        ThreadEvent::ThreadStarted(ThreadStarted {
            thread_id: expected.to_string(),
            extra: BTreeMap::new(),
        }),
        ThreadEvent::TurnStarted(TurnStarted {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            input_text: None,
            extra: BTreeMap::new(),
        }),
        ThreadEvent::TurnCompleted(TurnCompleted {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            last_item_id: None,
            extra: BTreeMap::new(),
        }),
        ThreadEvent::TurnFailed(TurnFailed {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            error: EventError {
                message: "fail".to_string(),
                code: None,
                extra: BTreeMap::new(),
            },
            extra: BTreeMap::new(),
        }),
        ThreadEvent::ItemStarted(ItemEnvelope {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            item: ItemSnapshot {
                item_id: "item-1".to_string(),
                index: None,
                status: ItemStatus::InProgress,
                payload: ItemPayload::AgentMessage(TextContent {
                    text: "".to_string(),
                    extra: BTreeMap::new(),
                }),
                extra: BTreeMap::new(),
            },
        }),
        ThreadEvent::ItemDelta(ItemDelta {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            item_id: "item-1".to_string(),
            index: None,
            delta: ItemDeltaPayload::AgentMessage(TextDelta {
                text_delta: "".to_string(),
                extra: BTreeMap::new(),
            }),
            extra: BTreeMap::new(),
        }),
        ThreadEvent::ItemCompleted(ItemEnvelope {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            item: ItemSnapshot {
                item_id: "item-1".to_string(),
                index: None,
                status: ItemStatus::Completed,
                payload: ItemPayload::AgentMessage(TextContent {
                    text: "".to_string(),
                    extra: BTreeMap::new(),
                }),
                extra: BTreeMap::new(),
            },
        }),
        ThreadEvent::ItemFailed(ItemEnvelope {
            thread_id: expected.to_string(),
            turn_id: "turn-1".to_string(),
            item: ItemFailure {
                item_id: "item-1".to_string(),
                index: None,
                error: EventError {
                    message: "fail".to_string(),
                    code: None,
                    extra: BTreeMap::new(),
                },
                extra: BTreeMap::new(),
            },
        }),
    ];

    for event in &events {
        assert_thread_id_borrowed(event, expected);
    }
}

#[test]
fn error_event_has_no_thread_id() {
    let event = ThreadEvent::Error(EventError {
        message: "boom".to_string(),
        code: None,
        extra: BTreeMap::new(),
    });

    assert!(event.thread_id().is_none());
}

#[test]
fn accessor_does_not_normalize_thread_id() {
    let raw = "  thread-123  ";
    let event = ThreadEvent::ThreadStarted(ThreadStarted {
        thread_id: raw.to_string(),
        extra: BTreeMap::new(),
    });

    assert_thread_id_borrowed(&event, raw);
}
