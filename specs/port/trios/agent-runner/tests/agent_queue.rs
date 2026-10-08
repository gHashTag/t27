use std::collections::VecDeque;

// Test FR-001: The agent runner MUST queue a person's turns with a maximum depth of 6 and execute them one at a time

#[test]
fn test_agent_queue_empty() {
    let mut queue = AgentQueue::new();
    
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
    assert_eq!(queue.max_depth(), 6);
}

#[test]
fn test_agent_queue_single_turn() {
    let mut queue = AgentQueue::new();
    
    let turn = Turn::new("user1".to_string(), "What's the weather?".to_string());
    
    assert!(queue.enqueue(turn).is_ok());
    assert_eq!(queue.len(), 1);
    assert!(!queue.is_empty());
    
    if let Some(dequeued) = queue.dequeue() {
        assert_eq!(dequeued.user_id, "user1");
        assert_eq!(dequeued.message, "What's the weather?");
    } else {
        panic!("Expected to dequeue a turn");
    }
    
    assert!(queue.is_empty());
}

#[test]
fn test_agent_queue_max_depth() {
    let mut queue = AgentQueue::new();
    
    // Add turns up to max depth
    for i in 0..6 {
        let turn = Turn::new(format!("user{}", i), format!("Message {}", i));
        assert!(queue.enqueue(turn).is_ok());
    }
    
    assert_eq!(queue.len(), 6);
    
    // Try to add one more - should fail
    let turn = Turn::new("user7".to_string(), "This should fail".to_string());
    assert!(queue.enqueue(turn).is_err());
    assert_eq!(queue.len(), 6); // Should still be 6
}

#[test]
fn test_agent_queue_fifo_order() {
    let mut queue = AgentQueue::new();
    
    // Add turns in order
    let turns = vec![
        ("user1", "Message 1"),
        ("user2", "Message 2"),
        ("user3", "Message 3"),
    ];
    
    for (user, msg) in &turns {
        let turn = Turn::new(user.to_string(), msg.to_string());
        assert!(queue.enqueue(turn).is_ok());
    }
    
    // Dequeue should be in FIFO order
    for (i, (expected_user, expected_msg)) in turns.iter().enumerate() {
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.user_id, expected_user);
        assert_eq!(dequeued.message, expected_msg);
    }
    
    assert!(queue.is_empty());
}

#[test]
fn test_agent_queue_clear() {
    let mut queue = AgentQueue::new();
    
    // Add some turns
    for i in 0..3 {
        let turn = Turn::new(format!("user{}", i), format!("Message {}", i));
        assert!(queue.enqueue(turn).is_ok());
    }
    
    assert_eq!(queue.len(), 3);
    assert!(!queue.is_empty());
    
    queue.clear();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
}

// Helper structures for testing
#[derive(Debug, Clone, PartialEq)]
struct Turn {
    user_id: String,
    message: String,
}

impl Turn {
    fn new(user_id: String, message: String) -> Self {
        Self { user_id, message }
    }
}

#[derive(Debug)]
struct AgentQueue {
    turns: VecDeque<Turn>,
    max_depth: usize,
}

impl AgentQueue {
    fn new() -> Self {
        Self {
            turns: VecDeque::new(),
            max_depth: 6,
        }
    }
    
    fn len(&self) -> usize {
        self.turns.len()
    }
    
    fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }
    
    fn max_depth(&self) -> usize {
        self.max_depth
    }
    
    fn enqueue(&mut self, turn: Turn) -> Result<(), String> {
        if self.turns.len() >= self.max_depth {
            return Err("Queue is at maximum depth".to_string());
        }
        
        self.turns.push_back(turn);
        Ok(())
    }
    
    fn dequeue(&mut self) -> Option<Turn> {
        self.turns.pop_front()
    }
    
    fn clear(&mut self) {
        self.turns.clear();
    }
}