use crate::model::Model;

pub type ActionCallback<'m> = fn(&mut Model<'m>);

/// Action triggered by the user.
pub struct Action<'m> {
  pub name: String,
  pub description: String,
  callback: ActionCallback<'m>,
}

impl<'m> Action<'m> {
  pub fn new(name: &str, description: &str, func: ActionCallback<'m>) -> Action<'m> {
    Action {
      name: name.to_string(),
      description: description.to_string(),
      callback: func,
    }
  }

  pub fn execute(&self, model: &mut Model<'m>) {
    (self.callback)(model);
  }
}

impl<'m> Clone for Action<'m> {
  fn clone(&self) -> Action<'m> {
    let name: String = self.name.clone();
    let description: String = self.description.clone();
    let callback: ActionCallback<'m> = self.callback;
    return Action {
      name: name,
      description: description,
      callback: callback,
    };
  }
}
