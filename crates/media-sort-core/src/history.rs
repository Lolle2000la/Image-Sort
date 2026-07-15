use crate::actions::reversible::{ActionError, ReversibleAction};

const MAX_HISTORY_SIZE: usize = 256;

pub struct History {
    done: Vec<Box<dyn ReversibleAction>>,
    undone: Vec<Box<dyn ReversibleAction>>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            done: Vec::with_capacity(64),
            undone: Vec::with_capacity(16),
        }
    }

    pub fn push_executed(&mut self, action: Box<dyn ReversibleAction>) {
        if self.done.len() >= MAX_HISTORY_SIZE {
            self.done.remove(0);
        }
        self.done.push(action);
        self.undone.clear();
    }

    pub fn undo(&mut self) -> Result<(), ActionError> {
        let mut action = self
            .done
            .pop()
            .ok_or_else(|| ActionError::RestorationFailed("nothing to undo".into()))?;
        action.rollback()?;
        self.undone.push(action);
        Ok(())
    }

    pub fn redo(&mut self) -> Result<(), ActionError> {
        let mut action = self
            .undone
            .pop()
            .ok_or_else(|| ActionError::RestorationFailed("nothing to redo".into()))?;
        action.execute()?;
        self.done.push(action);
        Ok(())
    }

    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }

    pub fn last_done_name(&self) -> Option<&str> {
        self.done.last().map(|a| a.display_name())
    }

    pub fn last_undone_name(&self) -> Option<&str> {
        self.undone.last().map(|a| a.display_name())
    }

    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
    pub fn done_len(&self) -> usize {
        self.done.len()
    }
    pub fn undone_len(&self) -> usize {
        self.undone.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::reversible::{ActionError, ReversibleAction};
    use crate::history::History;

    struct MockAction {
        name: String,
    }

    impl MockAction {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl ReversibleAction for MockAction {
        fn display_name(&self) -> &str {
            &self.name
        }

        fn execute(&mut self) -> Result<(), ActionError> {
            Ok(())
        }

        fn rollback(&mut self) -> Result<(), ActionError> {
            Ok(())
        }
    }

    struct FailingMockAction {
        name: String,
        execute_should_fail: bool,
        rollback_should_fail: bool,
    }

    impl FailingMockAction {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                execute_should_fail: false,
                rollback_should_fail: false,
            }
        }

        fn with_failing_rollback(mut self) -> Self {
            self.rollback_should_fail = true;
            self
        }

        fn with_failing_execute(mut self) -> Self {
            self.execute_should_fail = true;
            self
        }
    }

    impl ReversibleAction for FailingMockAction {
        fn display_name(&self) -> &str {
            &self.name
        }

        fn execute(&mut self) -> Result<(), ActionError> {
            if self.execute_should_fail {
                Err(ActionError::RestorationFailed("mock execute failed".into()))
            } else {
                Ok(())
            }
        }

        fn rollback(&mut self) -> Result<(), ActionError> {
            if self.rollback_should_fail {
                Err(ActionError::RestorationFailed(
                    "mock rollback failed".into(),
                ))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_push_and_query() {
        let mut history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.done_len(), 0);

        history.push_executed(Box::new(MockAction::new("test_action")));

        assert!(history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.done_len(), 1);
        assert_eq!(history.last_done_name(), Some("test_action"));
        assert_eq!(history.last_undone_name(), None);
    }

    #[test]
    fn test_undo_redo() {
        let mut history = History::new();
        history.push_executed(Box::new(MockAction::new("action1")));
        history.push_executed(Box::new(MockAction::new("action2")));

        history.undo().unwrap();
        assert_eq!(history.done_len(), 1);
        assert_eq!(history.undone_len(), 1);
        assert_eq!(history.last_done_name(), Some("action1"));
        assert_eq!(history.last_undone_name(), Some("action2"));
        assert!(history.can_undo());
        assert!(history.can_redo());

        history.redo().unwrap();
        assert_eq!(history.done_len(), 2);
        assert_eq!(history.undone_len(), 0);
        assert_eq!(history.last_done_name(), Some("action2"));
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_clear() {
        let mut history = History::new();
        history.push_executed(Box::new(MockAction::new("a")));
        history.push_executed(Box::new(MockAction::new("b")));
        history.push_executed(Box::new(MockAction::new("c")));

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.done_len(), 0);
        assert_eq!(history.undone_len(), 0);
    }

    #[test]
    fn test_overflow() {
        let mut history = History::new();
        for i in 0..260 {
            history.push_executed(Box::new(MockAction::new(&format!("action{}", i))));
        }

        assert_eq!(history.done_len(), 256);
        assert_eq!(history.undone_len(), 0);
        assert_eq!(history.last_done_name(), Some("action259"));
    }

    #[test]
    fn test_undo_on_empty() {
        let mut history = History::new();
        let result = history.undo();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ActionError::RestorationFailed(_)
        ));
    }

    #[test]
    fn test_redo_on_empty() {
        let mut history = History::new();
        let result = history.redo();
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ActionError::RestorationFailed(_)
        ));
    }

    #[test]
    fn test_redo_clears_on_push() {
        let mut history = History::new();
        history.push_executed(Box::new(MockAction::new("action1")));
        history.push_executed(Box::new(MockAction::new("action2")));

        history.undo().unwrap();
        assert_eq!(history.undone_len(), 1);
        assert_eq!(history.last_undone_name(), Some("action2"));

        history.push_executed(Box::new(MockAction::new("action3")));
        assert_eq!(history.undone_len(), 0);
        assert_eq!(history.last_undone_name(), None);
        assert!(!history.can_redo());
        assert_eq!(history.done_len(), 2);
    }

    #[test]
    fn test_history_max_boundary_exact() {
        let mut history = History::new();
        for i in 0..256 {
            history.push_executed(Box::new(MockAction::new(&format!("action{}", i))));
        }
        assert_eq!(history.done_len(), 256);
        assert_eq!(history.last_done_name(), Some("action255"));

        // Undo one, push another - should be at 256, not 257
        history.undo().unwrap();
        history.push_executed(Box::new(MockAction::new("action_after_undo")));
        assert_eq!(history.done_len(), 256);
        assert_eq!(history.undone_len(), 0);

        // Push one more to trigger overflow trimming - oldest entry should be dropped
        history.push_executed(Box::new(MockAction::new("overflow_action")));
        assert_eq!(history.done_len(), 256);
        assert_eq!(history.last_done_name(), Some("overflow_action"));
    }

    #[test]
    fn test_history_double_undo_second_fails() {
        let mut history = History::new();
        history.push_executed(Box::new(MockAction::new("only_one")));
        assert!(history.can_undo());

        // First undo should succeed
        history.undo().unwrap();
        assert!(!history.can_undo());
        assert_eq!(history.done_len(), 0);

        // Second undo should fail
        let result = history.undo();
        assert!(result.is_err());
    }

    #[test]
    fn test_history_double_redo_second_fails() {
        let mut history = History::new();
        history.push_executed(Box::new(MockAction::new("action")));
        history.undo().unwrap();
        assert!(history.can_redo());

        // First redo should succeed
        history.redo().unwrap();
        assert!(!history.can_redo());

        // Second redo should fail
        let result = history.redo();
        assert!(result.is_err());
    }

    #[test]
    fn test_history_last_done_name_empty() {
        let history = History::new();
        assert_eq!(history.last_done_name(), None);
    }

    #[test]
    fn test_history_interleaved_undo_redo() {
        let mut history = History::new();
        let mut mock = MockAction::new("A");
        mock.execute().unwrap();
        history.push_executed(Box::new(mock));
        let mut mock2 = MockAction::new("B");
        mock2.execute().unwrap();
        history.push_executed(Box::new(mock2));
        let mut mock3 = MockAction::new("C");
        mock3.execute().unwrap();
        history.push_executed(Box::new(mock3));

        assert_eq!(history.done_len(), 3);

        history.undo().unwrap();
        assert_eq!(history.done_len(), 2);
        assert_eq!(history.undone_len(), 1);

        history.undo().unwrap();
        assert_eq!(history.done_len(), 1);
        assert_eq!(history.undone_len(), 2);

        history.redo().unwrap();
        assert_eq!(history.done_len(), 2);
        assert_eq!(history.undone_len(), 1);

        history.undo().unwrap();
        assert_eq!(history.done_len(), 1);
        assert_eq!(history.undone_len(), 2);
    }

    #[test]
    fn test_history_undo_failing_rollback() {
        let mut history = History::new();
        let action = FailingMockAction::new("fail_rollback").with_failing_rollback();
        history.push_executed(Box::new(action));

        assert!(history.can_undo());
        let result = history.undo();
        assert!(result.is_err());
        // The action was popped from done, rollback failed, so it was NOT pushed to undone
        assert!(!history.can_undo());
        assert!(!history.can_redo());
        assert_eq!(history.done_len(), 0);
        assert_eq!(history.undone_len(), 0);
    }

    #[test]
    fn test_history_redo_failing_execute() {
        let mut history = History::new();
        // Push two actions: one good, one whose execute will fail on redo
        history.push_executed(Box::new(MockAction::new("good")));
        history.push_executed(Box::new(
            FailingMockAction::new("fail_execute").with_failing_execute(),
        ));
        // Undo the failing-execute action (rollback succeeds)
        history.undo().unwrap();
        assert!(history.can_redo());

        let result = history.redo();
        // redo pops from undone, calls execute which fails
        assert!(result.is_err());
        // The action was popped from undone, execute failed, not pushed to done
        assert!(!history.can_redo());
        assert_eq!(history.undone_len(), 0);
        assert_eq!(history.done_len(), 1); // "good" is still in done
    }
}
