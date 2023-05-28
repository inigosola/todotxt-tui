use std::collections::BTreeSet;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::str::FromStr;
use todo_txt::Task;

#[allow(dead_code)]
pub struct ToDo {
    pending: Vec<Task>,
    done: Vec<Task>,
    use_done: bool,
}

#[allow(dead_code)]
impl ToDo {
    pub fn load<R>(reader: R, use_done: bool) -> Result<ToDo, Box<dyn Error>>
    where
        R: Read,
    {
        let mut pending = Vec::new();
        let mut done = Vec::new();
        for line in BufReader::new(reader).lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let task = Task::from_str(&line)?;
            if task.finished {
                done.push(task);
            } else {
                pending.push(task);
            }
        }

        Ok(ToDo {
            pending,
            done,
            use_done,
        })
    }

    fn get_btree(tasks: Vec<&Vec<Task>>, f: fn(&Task) -> &Vec<String>) -> BTreeSet<String> {
        let mut btree = BTreeSet::new();

        tasks.iter().for_each(|list| {
            list.iter().for_each(|task| {
                f(task).iter().for_each(|project| {
                    btree.insert(project.clone());
                })
            })
        });
        btree
    }

    fn get_btree_done_switch(&self, f: fn(&Task) -> &Vec<String>) -> BTreeSet<String> {
        Self::get_btree(
            if self.use_done {
                vec![&self.pending, &self.done]
            } else {
                vec![&self.pending]
            },
            f,
        )
    }

    pub fn get_projects(&self) -> BTreeSet<String> {
        self.get_btree_done_switch(|t| &t.projects)
    }

    pub fn get_contexts(&self) -> BTreeSet<String> {
        self.get_btree_done_switch(|t| &t.contexts)
    }

    pub fn get_hashtags(&self) -> BTreeSet<String> {
        self.get_btree_done_switch(|t| &t.hashtags)
    }

    fn get_tasks<'a>(
        tasks: Vec<&'a Vec<Task>>,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
    ) -> Vec<&'a Task> {
        let mut vec = Vec::new();
        tasks.iter().for_each(|list| {
            vec.append(
                &mut list
                    .iter()
                    .filter(|task| f(task).contains(&String::from(name)))
                    .collect::<Vec<&'a Task>>(),
            );
        });
        vec
    }

    fn get_tasks_done_switch<'a>(
        &'a self,
        name: &str,
        f: fn(&Task) -> &Vec<String>,
    ) -> Vec<&'a Task> {
        Self::get_tasks(
            if self.use_done {
                vec![&self.pending, &self.done]
            } else {
                vec![&self.pending]
            },
            name,
            f,
        )
    }

    pub fn get_project_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.projects)
    }

    pub fn get_context_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.contexts)
    }

    fn get_hashtag_tasks<'a>(&'a self, name: &str) -> Vec<&'a Task> {
        self.get_tasks_done_switch(name, |t| &t.hashtags)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    const TESTING_STRING: &str = r#"
        x (A) 2023-05-21 2023-04-30 measure space for 1 +project1 @context1 #hashtag1 due:2023-06-30
                         2023-04-30 measure space for 2 +project2 @context2           due:2023-06-30
                     (C) 2023-04-30 measure space for 3 +project3 @context3           due:2023-06-30
                                    measure space for 4 +project2 @context3 #hashtag1 due:2023-06-30
                                  x measure space for 5 +project3 @context3 #hashtag2 due:2023-06-30
                                    measure space for 6 +project3 @context2 #hashtag2 due:2023-06-30
        "#;

    #[test]
    fn test_load() -> Result<(), Box<dyn Error>> {
        let todo = ToDo::load(TESTING_STRING.as_bytes(), true)?;

        assert_eq!(todo.done.len(), 2);
        assert_eq!(todo.pending.len(), 4);

        assert_eq!(todo.done[0].priority, 0);
        assert!(todo.done[0].create_date.is_some());
        assert!(todo.done[0].finish_date.is_some());
        assert_eq!(todo.done[0].finished, true);
        assert_eq!(todo.done[0].threshold_date, None);
        assert!(todo.done[0].due_date.is_some());
        assert_eq!(todo.done[0].contexts.len(), 1);
        assert_eq!(todo.done[0].projects.len(), 1);
        assert_eq!(todo.done[0].hashtags.len(), 1);

        assert!(todo.pending[0].priority.is_lowest());
        assert!(todo.pending[0].create_date.is_some());
        assert!(todo.pending[0].finish_date.is_none());
        assert_eq!(todo.pending[0].finished, false);
        assert_eq!(todo.pending[0].threshold_date, None);
        assert!(todo.pending[0].due_date.is_some());
        assert_eq!(todo.pending[0].contexts.len(), 1);
        assert_eq!(todo.pending[0].projects.len(), 1);
        assert_eq!(todo.pending[0].hashtags.len(), 0);

        assert_eq!(todo.pending[1].priority, 2);
        assert!(todo.pending[1].create_date.is_some());
        assert!(todo.pending[1].finish_date.is_none());
        assert_eq!(todo.pending[1].finished, false);
        assert_eq!(todo.pending[1].threshold_date, None);
        assert!(todo.pending[1].due_date.is_some());
        assert_eq!(todo.pending[1].contexts.len(), 1);
        assert_eq!(todo.pending[1].projects.len(), 1);
        assert_eq!(todo.pending[1].hashtags.len(), 0);

        Ok(())
    }

    #[test]
    fn test_categeries_list() -> Result<(), Box<dyn Error>> {
        let create_btree = |items: &[&str]| {
            let mut btree: BTreeSet<String> = BTreeSet::new();
            items.iter().for_each(|item| {
                btree.insert(item.to_string());
            });
            btree
        };

        let mut todo = ToDo::load(TESTING_STRING.as_bytes(), false)?;
        assert_eq!(todo.get_projects(), create_btree(&["project2", "project3"]));
        assert_eq!(todo.get_contexts(), create_btree(&["context2", "context3"]));
        assert_eq!(todo.get_hashtags(), create_btree(&["hashtag1", "hashtag2"]));

        todo.use_done = true;
        assert_eq!(
            todo.get_projects(),
            create_btree(&["project1", "project2", "project3"])
        );
        assert_eq!(
            todo.get_contexts(),
            create_btree(&["context1", "context2", "context3"])
        );
        assert_eq!(todo.get_hashtags(), create_btree(&["hashtag1", "hashtag2"]));

        Ok(())
    }

    #[test]
    fn test_tasks_in_category() -> Result<(), Box<dyn Error>> {
        let mut todo = ToDo::load(TESTING_STRING.as_bytes(), false)?;
        assert_eq!(todo.get_project_tasks("project1").len(), 0);
        assert_eq!(todo.get_project_tasks("project2").len(), 2);
        assert_eq!(todo.get_project_tasks("project3").len(), 2);
        assert_eq!(todo.get_context_tasks("context1").len(), 0);
        assert_eq!(todo.get_context_tasks("context2").len(), 2);
        assert_eq!(todo.get_context_tasks("context3").len(), 2);
        assert_eq!(todo.get_hashtag_tasks("hashtag1").len(), 1);
        assert_eq!(todo.get_hashtag_tasks("hashtag2").len(), 1);

        todo.use_done = true;
        assert_eq!(todo.get_project_tasks("project1").len(), 1);
        assert_eq!(todo.get_project_tasks("project2").len(), 2);
        assert_eq!(todo.get_project_tasks("project3").len(), 3);
        assert_eq!(todo.get_context_tasks("context1").len(), 1);
        assert_eq!(todo.get_context_tasks("context2").len(), 2);
        assert_eq!(todo.get_context_tasks("context3").len(), 3);
        assert_eq!(todo.get_hashtag_tasks("hashtag1").len(), 2);
        assert_eq!(todo.get_hashtag_tasks("hashtag2").len(), 2);

        Ok(())
    }
}
