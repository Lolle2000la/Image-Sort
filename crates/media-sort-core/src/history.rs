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
