use model::SharedModel;

pub type ActionCallback<'m> = fn(&mut SharedModel<'m>);

/// Action triggered by the user.
pub struct Action<'m> {
  name: String,
  callback: ActionCallback<'m>,
}

impl<'m> Action<'m> {
  pub fn new(name: &str, func: ActionCallback<'m>) -> Action<'m> {
    Action {
      name: name.to_string(),
      callback: func,
    }
  }

  pub fn execute(&self, model: &mut SharedModel<'m>) {
    (self.callback)(model);
  }
}

impl<'m> Clone for Action<'m> {
  fn clone(&self) -> Action<'m> {
    let name: String = self.name.clone();
    let callback: ActionCallback<'m> = self.callback;
    return Action {
      name: name,
      callback: callback,
    };
  }
}
