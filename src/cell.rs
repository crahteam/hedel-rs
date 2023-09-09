use std::{
	fmt::Debug,
	ops::{
		Deref,
		DerefMut
	},
	cell::{
		Cell,
		UnsafeCell
	},
	ptr::NonNull,
	num::NonZeroUsize
};

use crate::errors::HedelError;

#[derive(Debug, Clone, Copy)]
pub enum BorrowFlag {
	/// This flag indicates that a mutable reference is in scope.
	Exclusive, 
	/// This flag indicates that one or more immutable references are in scope.
	/// Contains the number of references alive.
	Shared(NonZeroUsize),
	/// No reference is in scope.
	None
}

/// A safe custom `RefCell-like` cell, based on `UnsafeCell`, and relying on a `BorrowFlag`
/// for runtime borrow checking.
#[derive(Debug)]
pub struct HedelCell<T: Debug> {
	flag: Cell<BorrowFlag>,
	cell: UnsafeCell<T>
}

impl<T: Debug> HedelCell<T> {

	/// The default constructor for `HedelCell`.
	///
	/// # Example
	///
	/// ```
	/// use hedel_rs::cell::HedelCell;
	///
	/// fn main() {
	/// 	let cell = HedelCell::<i32>::new(67);
	///		println!("{:?}", cell.get());
	/// }
	/// ```
	pub fn new(value: T) -> Self {
		Self {
			flag: Cell::new(BorrowFlag::None),
			cell: UnsafeCell::<T>::new(value)
		}
	}

	/// Get a `RefHedel` pointing to the inner value in a `HedelCell`.
	///
	/// SAFETY: checks if a mutable borrow is active and panics. Also increments 
	/// a shared reference counter.
	///
	/// # Example
	///
	/// ```
	/// use hedel_rs::cell::HedelCell;
	///
	/// fn main() {
	///		let cell = HedelCell::<i32>::new(56);
	///		let borrow = cell.get();
	///		let borrow_2 = cell.get();
	///		println!("{:?}", borrow); // prints 56
	/// }
	/// ```
	pub fn try_get(&self) -> Result<RefHedel<T>, HedelError> {
		
		match self.flag.get() {
			BorrowFlag::None => {
				self.flag.replace(BorrowFlag::Shared(NonZeroUsize::new(1).unwrap()));
			},
			BorrowFlag::Shared(n) => {
				self.flag.replace(BorrowFlag::Shared(n.saturating_add(1)));
			},
			_ => {
				return Err(HedelError::SharedBorrow);
			}
		}

		Ok(RefHedel {
			value: unsafe { &*self.cell.get() },
			flag: &self.flag
		})
	}
	
	/// Guarantees to return `RefHedel` or panics!
	pub fn get(&self) -> RefHedel<T> {
		self.try_get().unwrap()
	}

	/// Get a `RefMutHedel` mutably pointing to the inner value in a `HedelCell`.
	///
	/// SAFETY: panics when a mutable reference is alive or when there's one or more shared references.
	/// Also sets the flag to `BorrowFlag::Exclusive`.
	///
	/// # Example
	/// 
	/// ```
	/// use hedel_rs::cell::HedelCell;
	///
	/// fn main() {
	///		
	/// 	let cell = HedelCell::<i32>::new(23);
	/// 	*cell.get_mut() = 36;
	/// 	let mut borrow = cell.get_mut();
	/// 	*borrow = 15;
	///		// this would panic
	/// 	// println!("{:?}", cell.get()); 
	///	}
	 
	/// ```
	pub fn try_get_mut<'a>(&'a self) -> Result<RefMutHedel<'a, T>, HedelError> {
		if let BorrowFlag::None = self.flag.get() {

			self.flag.replace(BorrowFlag::Exclusive);

			let value = match NonNull::<T>::new(UnsafeCell::raw_get(&self.cell as *const UnsafeCell::<T>)) {
				Some(value) => value,
				None => return Err(HedelError::InvalidNonNull) 
			};

			return Ok(RefMutHedel::<T> {
				flag: &self.flag,
				value 
			});
		} Err(HedelError::MutBorrow_)
	}

	/// Guarantees to return `RefMutHedel` or panics!
	pub fn get_mut(&self) -> RefMutHedel<T> {
		self.try_get_mut().unwrap()
	}

	/// Consumes itself and returns the inner value
	pub fn into_inner(self) -> T {
		self.cell.into_inner()
	}
}

/// Represents an immutable reference to the content in a `HedelCell`.
/// Has to be built by calling `HedelCell::get`.
#[derive(Debug)]
pub struct RefHedel<'a, T: Debug> {
	value: &'a T,
	flag: &'a Cell<BorrowFlag>
}

/// Automatically dereferences `RefHedel` to &T.
impl<'a, T: Debug> Deref for RefHedel<'a, T> {
	type Target = T;
	fn deref(&self) -> &T {
		self.value
	}
}

/// SAFETY: when a `RefHedel` is dropped, the shared reference counter
/// is diminished by 1. To prevent it to reach 0 it is set to None.
impl<'a, T: Debug> Drop for RefHedel<'a, T> {
	fn drop(&mut self) {
		match self.flag.get() {
			BorrowFlag::Shared(n) => {
				if n.get() > 1 {
					self.flag.replace(BorrowFlag::Shared(NonZeroUsize::new(n.get() - 1).unwrap()));
				} else {
					self.flag.replace(BorrowFlag::None);
				}
			},
			_ => {
				unreachable!("Before a `RefHedel` gets dropped, there should be a `BorrowFlag::Shared(_)`");
			}
		}
	}
}

/// Represents a mutable reference to a `HedelCell`.
/// Has to be built by calling `HedelCell::get`.
pub struct RefMutHedel<'a, T: Debug> {
	value: NonNull<T>,
	flag: &'a Cell<BorrowFlag>
}

/// Automatically dereferences `RefMutHedel` to &T.
impl<'a, T: Debug> Deref for RefMutHedel<'a, T> {
	type Target = T;
	
	fn deref(&self) -> &T {
		unsafe { self.value.as_ref() } 
	}
}

/// Automatically dereferences `RefMutHedel` to &mut T.
impl<'a, T: Debug> DerefMut for RefMutHedel<'a, T> {

    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.value.as_mut() }
    }
}

/// SAFETY: before `RefMutHedel` gets dropped, it changes the flag to `BorrowFlag::None`,
/// meaning that now, shared immutable references are avaiable.
impl<'a, T: Debug> Drop for RefMutHedel<'a, T> {
	fn drop(&mut self) {
		self.flag.replace(BorrowFlag::None);
	}
}
