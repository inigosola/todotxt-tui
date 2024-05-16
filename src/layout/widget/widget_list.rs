use super::{RCToDo, WidgetBase, WidgetType};
use crate::config::Config;
use crate::ui::{EventHandlerUI, HandleEvent, UIEvent};
use crossterm::event::KeyCode;
use std::ops::{Deref, DerefMut};
use tui::widgets::ListState;

/// Represents a widget that displays a list of items.
pub struct WidgetList {
    base: WidgetBase,
    state: ListState,
    pub len: usize,
    first: usize,
    size: usize,
    event_handler: EventHandlerUI,
    list_shift: usize,
}

impl WidgetList {
    /// Creates a new `WidgetList` instance.
    ///
    /// # Parameters
    ///
    /// - `widget_type`: The type of widget.
    /// - `data`: A reference-counted mutex of `ToDo` data.
    ///
    /// # Returns
    ///
    /// A new `WidgetList` instance.
    pub fn new(widget_type: &WidgetType, data: RCToDo, config: &Config) -> Self {
        let mut def = Self {
            base: WidgetBase::new(widget_type, data, config),
            state: ListState::default(),
            len: 0,
            first: 0,
            size: 0,
            event_handler: config.get_list_keybind(),
            list_shift: config.get_list_shift(),
        };
        def.state.select(Some(0));
        def
    }

    /// Gets the currently selected item index.
    ///
    /// # Returns
    ///
    /// The index of the selected item.
    pub fn act(&self) -> usize {
        self.state.selected().unwrap_or(0)
    }

    /// Gets the index of the item within the entire list, accounting for the first visible item.
    ///
    /// # Returns
    ///
    /// The adjusted index of the item.
    pub fn index(&self) -> usize {
        self.act() + self.first
    }

    /// Gets a clone of the list state.
    ///
    /// # Returns
    ///
    /// A clone of the list state.
    pub fn state(&self) -> ListState {
        self.state.clone()
    }

    /// Sets the size of the list widget.
    ///
    /// # Parameters
    ///
    /// - `size`: The size of the list widget.
    pub fn set_size(&mut self, size: u16) {
        self.size = size as usize;
    }

    /// Moves the selection down the list.
    pub fn down(&mut self) {
        let act = self.act();
        if self.len <= self.size {
            if self.len > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else if self.size <= act + 1 + self.list_shift {
            if self.first + self.size < self.len {
                self.first += 1;
            } else if self.size > act + 1 {
                self.state.select(Some(act + 1));
            }
        } else {
            self.state.select(Some(act + 1));
        }
        log::trace!(
            "List go down: act: {}, size: {} len: {}, shift: {}",
            act,
            self.size,
            self.len,
            self.list_shift
        );
    }

    /// Moves the selection up the list.
    pub fn up(&mut self) {
        let act = self.act();
        if act <= self.list_shift {
            if self.first > 0 {
                self.first -= 1;
            } else if act > 0 {
                self.state.select(Some(act - 1));
            }
        } else {
            self.state.select(Some(act - 1));
        }
        log::trace!("List go up: act: {}", act);
    }

    /// Moves the selection to the next item and returns
    /// the indices of the old and new selections.
    ///
    /// # Returns
    ///
    /// An `Option` containing the indices of the (old, new) selections,
    /// or `None` if the list is at the end.
    pub fn next(&mut self) -> Option<(usize, usize)> {
        log::error!("len: {}, index: {}", self.len, self.index());
        if self.len <= self.index() + 1 {
            None
        } else {
            let old = self.index();
            self.down();
            Some((old, self.index()))
        }
    }

    /// Moves the selection to the previous item and returns the indices
    /// of the old and new selections.
    ///
    /// # Returns
    ///
    /// An `Option` containing the indices of the old and new selections,
    /// or `None` if the list is at the beginning.
    pub fn prev(&mut self) -> Option<(usize, usize)> {
        if self.act() == 0 {
            None
        } else {
            let old = self.index();
            self.up();
            Some((old, self.index()))
        }
    }

    /// Moves the selection to the first item in the list.
    pub fn first(&mut self) {
        self.state.select(Some(0));
        self.first = 0;
    }

    /// Moves the selection to the last item in the list.
    pub fn last(&mut self) {
        let shown_items = self.len - 1;
        if self.size > shown_items {
            self.first = 0;
            self.state.select(Some(shown_items));
        } else {
            self.first = self.len - self.size;
            self.state.select(Some(self.size - 1));
        }
    }

    /// Gets the range of items currently displayed in the list.
    ///
    /// # Returns
    ///
    /// A tuple containing the indices of the (first, last) items displayed.
    pub fn range(&self) -> (usize, usize) {
        (self.first, self.first + self.size)
    }
}

impl HandleEvent for WidgetList {
    fn get_event(&self, key: &KeyCode) -> UIEvent {
        self.event_handler.get_event(key)
    }

    fn handle_event(&mut self, event: UIEvent) -> bool {
        match event {
            UIEvent::ListDown => self.down(),
            UIEvent::ListUp => self.up(),
            UIEvent::ListFirst => self.first(),
            UIEvent::ListLast => self.last(),
            _ => return false,
        }
        true
    }
}

impl Deref for WidgetList {
    type Target = WidgetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl DerefMut for WidgetList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::todo::ToDo;
    use std::sync::{Arc, Mutex};
    use test_log::test;

    fn testing_widget(len: usize) -> WidgetList {
        let mut todo = ToDo::default();
        for i in 1..len {
            todo.new_task(&format!("Task {}", i)).unwrap();
        }
        let todo = Arc::new(Mutex::new(todo));
        let mut widget = WidgetList::new(&WidgetType::List, todo, &Config::default());
        widget.set_size(10);
        widget.len = len;

        widget
    }

    fn n_times(times: usize, func: fn(&mut WidgetList), s: &mut WidgetList) {
        for _ in 0..times {
            func(s)
        }
    }

    #[test]
    fn movement_in_short_list() {
        let mut widget = testing_widget(5);

        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);

        widget.down();
        assert_eq!(widget.index(), 1);
        assert_eq!(widget.act(), 1);
        assert_eq!(widget.first, 0);
    }

    #[test]
    fn movement_basic() {
        let mut widget = testing_widget(50);

        // Starting position
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);

        // First down
        widget.down();
        assert_eq!(widget.index(), 1);
        assert_eq!(widget.act(), 1);
        assert_eq!(widget.first, 0);

        // Second down
        widget.down();
        assert_eq!(widget.index(), 2);
        assert_eq!(widget.act(), 2);
        assert_eq!(widget.first, 0);

        // First up
        widget.up();
        assert_eq!(widget.index(), 1);
        assert_eq!(widget.act(), 1);
        assert_eq!(widget.first, 0);

        // Third down
        widget.down();
        assert_eq!(widget.index(), 2);
        assert_eq!(widget.act(), 2);
        assert_eq!(widget.first, 0);
    }

    #[test]
    fn movement_full_list() {
        let mut widget = testing_widget(50);

        // Before first full list move
        n_times(5, WidgetList::down, &mut widget);

        assert_eq!(widget.index(), 5);
        assert_eq!(widget.act(), 5);
        assert_eq!(widget.first, 0);

        // First full list move
        widget.down();

        assert_eq!(widget.index(), 6);
        assert_eq!(widget.act(), 5);
        assert_eq!(widget.first, 1);

        // Second full list move
        widget.down();

        assert_eq!(widget.index(), 7);
        assert_eq!(widget.act(), 5);
        assert_eq!(widget.first, 2);

        // Move to last item
        n_times(50, WidgetList::down, &mut widget);
        assert_eq!(widget.index(), 49);
        assert_eq!(widget.act(), 9);
        assert_eq!(widget.first, 40);

        // Move up
        widget.up();
        assert_eq!(widget.index(), 48);
        assert_eq!(widget.act(), 8);
        assert_eq!(widget.first, 40);

        // Before first full list move up
        n_times(4, WidgetList::up, &mut widget);
        assert_eq!(widget.index(), 44);
        assert_eq!(widget.act(), 4);
        assert_eq!(widget.first, 40);

        // First full list move up
        widget.up();
        assert_eq!(widget.index(), 43);
        assert_eq!(widget.act(), 4);
        assert_eq!(widget.first, 39);

        // Go to start of the list where full list stop moving
        n_times(39, WidgetList::up, &mut widget);
        assert_eq!(widget.index(), 4);
        assert_eq!(widget.act(), 4);
        assert_eq!(widget.first, 0);

        widget.up();
        assert_eq!(widget.index(), 3);
        assert_eq!(widget.act(), 3);
        assert_eq!(widget.first, 0);

        // Go to first index
        n_times(3, WidgetList::up, &mut widget);
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);
    }

    #[test]
    fn move_task() {
        let mut widget = testing_widget(50);
        assert_eq!(widget.next(), Some((0, 1)));
        assert_eq!(widget.next(), Some((1, 2)));
        assert_eq!(widget.next(), Some((2, 3)));
        assert_eq!(widget.next(), Some((3, 4)));
        assert_eq!(widget.next(), Some((4, 5)));

        assert_eq!(widget.prev(), Some((5, 4)));
        assert_eq!(widget.prev(), Some((4, 3)));
        assert_eq!(widget.prev(), Some((3, 2)));
        assert_eq!(widget.prev(), Some((2, 1)));
        assert_eq!(widget.prev(), Some((1, 0)));

        widget.down();
        assert_eq!(widget.next(), Some((1, 2)));

        widget.up();
        assert_eq!(widget.next(), Some((1, 2)));

        widget.up();
        assert_eq!(widget.next(), Some((1, 2)));
    }

    #[test]
    fn move_task_borders() {
        let mut widget = testing_widget(50);
        assert_eq!(widget.prev(), None);

        widget.down();
        assert_eq!(widget.prev(), Some((1, 0)));

        n_times(50, WidgetList::down, &mut widget);
        assert_eq!(widget.next(), None);

        widget.up();
        assert_eq!(widget.next(), Some((48, 49)));
    }

    #[test]
    fn first_and_last_item() {
        // Long list
        let mut widget = testing_widget(50);
        widget.last();
        assert_eq!(widget.index(), 49);
        assert_eq!(widget.act(), 9);
        assert_eq!(widget.first, 40);

        widget.first();
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);

        // Short list
        let mut widget = testing_widget(5);
        widget.last();
        assert_eq!(widget.index(), 4);
        assert_eq!(widget.act(), 4);
        assert_eq!(widget.first, 0);

        widget.first();
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);
    }

    #[test]
    fn range() {
        let widget = testing_widget(50);
        assert_eq!(widget.range(), (0, 10));
    }

    #[test]
    fn handle_event() {
        let mut widget = testing_widget(50);
        assert!(widget.handle_event(UIEvent::ListDown));
        assert_eq!(widget.index(), 1);
        assert_eq!(widget.act(), 1);
        assert_eq!(widget.first, 0);

        assert!(widget.handle_event(UIEvent::ListUp));
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);

        assert!(widget.handle_event(UIEvent::ListLast));
        assert_eq!(widget.index(), 49);
        assert_eq!(widget.act(), 9);
        assert_eq!(widget.first, 40);

        assert!(widget.handle_event(UIEvent::ListFirst));
        assert_eq!(widget.index(), 0);
        assert_eq!(widget.act(), 0);
        assert_eq!(widget.first, 0);

        assert!(!widget.handle_event(UIEvent::None));
    }
}
