use crate::CONFIG;
use serde::{Deserialize, Serialize};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, Paragraph},
    Frame,
};
use crate::todo::ToDo;
use std::rc::Rc;

#[allow(dead_code)]
#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Input,
    List,
    Done,
    Project,
    Context,
}

pub struct Widget {
    pub widget_type: WidgetType,
    pub chunk: Rect,
    pub title: String,
    pub data: Rc<ToDo>,
}

impl Widget {
    pub fn new(widget_type: WidgetType, title: &str, data: Rc<ToDo>) -> Widget {
        Widget {
            widget_type,
            chunk: Rect {
                width: 0,
                height: 0,
                x: 0,
                y: 0,
            },
            title: title.to_string(),
            data,
        }
    }

    pub fn update_chunk(&mut self, chunk: Rect) {
        self.chunk = chunk;
    }

    pub fn draw<B>(&self, f: &mut Frame<B>, active: bool)
    where
        B: Backend,
    {
        let get_block = || {
            let mut block = Block::default()
                .borders(Borders::ALL)
                .title(self.title.clone())
                .border_type(BorderType::Rounded);
            if active {
                block = block.border_style(Style::default().fg(CONFIG.active_color));
            }
            block
        };

        match self.widget_type {
            WidgetType::Input => {
                f.render_widget(Paragraph::new("Some text").block(get_block()), self.chunk);
            }
            WidgetType::List => {
                let list = List::new(self.data.pending.clone())
                    .block(get_block())
                    .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                    .highlight_symbol(">>");
                f.render_widget(list, self.chunk);
            }
            WidgetType::Done => {
                let list = List::new(self.data.done.clone())
                    .block(get_block())
                    .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                    .highlight_symbol(">>");
                f.render_widget(list, self.chunk);
            }
            WidgetType::Project => {
                let list = List::new(self.data.get_projects())
                    .block(get_block())
                    .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                    .highlight_symbol(">>");
                f.render_widget(list, self.chunk);
            }
            WidgetType::Context => {
                f.render_widget(get_block(), self.chunk);
            }
        }
    }
}
