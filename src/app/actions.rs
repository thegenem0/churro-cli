use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    slice::Iter,
};

use crate::inputs::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    Sleep,
    IncrementDelay,
    DecrementDelay,
}

impl Action {
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 4] = [
            Action::Quit,
            Action::Sleep,
            Action::IncrementDelay,
            Action::DecrementDelay,
        ];
        ACTIONS.iter()
    }

    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Char('q'), Key::Ctrl('c')],
            Action::Sleep => &[Key::Char('s')],
            Action::IncrementDelay => &[Key::Char('i')],
            Action::DecrementDelay => &[Key::Char('d')],
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Action::Quit => write!(f, "Quit"),
            Action::Sleep => write!(f, "Sleep"),
            Action::IncrementDelay => write!(f, "Increment Delay"),
            Action::DecrementDelay => write!(f, "Decrement Delay"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Actions(Vec<Action>);

impl Actions {
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}
