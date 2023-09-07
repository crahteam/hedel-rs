use thiserror::Error;

#[derive(Error, Debug)]
pub enum HedelError {
	#[error("There already is a mutable reference alive to `HedelCell`.
	Getting another mutable reference to it is Undefined Behavior.")]
	MutBorrow,
	#[error("There are one or more shared references alive to `HedelCell`.
	Getting a mutable reference to it is Undefined Behavior.")]
	MutBorrow_,
	#[error("There is a mutable reference alive to `HedelCell`.
	Getting a shared reference to it is Undefined Behavior.")]
	SharedBorrow,
	#[error("A `NonNull` pointer to the value in HedelCell was null.")]
	InvalidNonNull
}
