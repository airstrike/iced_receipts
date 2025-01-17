//! Common API for returning either Tasks or generic Operations from views.
use iced::advanced::graphics::futures::MaybeSend;
use iced::Task;
use std::fmt;

pub struct Action<Operation, Message> {
    pub operation: Option<Operation>,
    pub task: Task<Message>,
}

impl<Operation, Message> Action<Operation, Message> {
    pub fn none() -> Self {
        Self {
            operation: None,
            task: Task::none(),
        }
    }

    pub fn new(operation: Operation, task: Task<Message>) -> Self {
        Self {
            operation: Some(operation),
            task,
        }
    }

    /// Create a new Action with an [`Operation`](Operation) to be handled
    /// by some ancestor component.
    pub fn operation(operation: Operation) -> Self {
        Self {
            operation: Some(operation),
            task: Task::none(),
        }
    }

    /// Create a new Action with a [`Task`](iced::Task).
    pub fn task(task: Task<Message>) -> Self {
        Self {
            operation: None,
            task,
        }
    }

    /// Map the message of the Action's [`Task`](iced::Task) to a different type.
    pub fn map<N>(self, f: impl Fn(Message) -> N + MaybeSend + 'static) -> Action<Operation, N>
    where
        Message: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            operation: self.operation,
            task: self.task.map(f),
        }
    }

    pub fn map_operation<N>(
        self,
        f: impl Fn(Operation) -> N + MaybeSend + 'static,
    ) -> Action<N, Message>
    where
        Operation: MaybeSend + 'static,
        N: MaybeSend + 'static,
    {
        Action {
            operation: self.operation.map(f),
            task: self.task,
        }
    }

    pub fn with_operation(mut self, operation: Operation) -> Self {
        self.operation = Some(operation);
        self
    }

    pub fn with_task(mut self, task: Task<Message>) -> Self {
        self.task = task;
        self
    }
}

impl<Operation: fmt::Debug, Message> fmt::Debug for Action<Operation, Message> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Action")
            .field("operation", &self.operation)
            .finish()
    }
}
