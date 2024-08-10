use super::location::Location;

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext {
  messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
  message: String,
  message_type: MessageType,
  location: Location,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageType {
  Warning,
  Error,
}

impl ErrorContext {
  pub fn new() -> Self {
    ErrorContext {
      messages: Vec::new(),
    }
  }

  pub fn merge(&mut self, mut other: ErrorContext) {
    self.messages.append(&mut other.messages);
  }

  pub fn add_message(&mut self, message: Message) {
    self.messages.push(message);
  }

  pub fn add_error(&mut self, message: String, location: Location) {
    self.add_message(Message {
      message,
      message_type: MessageType::Error,
      location,
    });
  }
  pub fn add_warning(&mut self, message: String, location: Location) {
    self.add_message(Message {
      message,
      message_type: MessageType::Warning,
      location,
    });
  }
}
