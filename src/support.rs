use std::{ffi::CStr, marker::PhantomData, mem::size_of};

pub type Opaque = [u8; 0];

pub fn cstr_to_string(cstr: &CStr) -> String {
  String::from_utf8_lossy(cstr.to_bytes()).into_owned()
}

#[doc(hidden)]
#[macro_export]
macro_rules! check_exception {
  ($value: expr, $ctx: expr) => {{
    let value = $value;
    let ctx = $ctx;
    if value.is_exception() {
      let e = ctx.get_exception();
      Err($crate::error::JSContextException::from_jsvalue(ctx, e))
    } else {
      Ok(value)
    }
  }};
}

// from: https://github.com/denoland/rusty_v8/pull/207
pub trait UnitType
where
  Self: Copy + Sized,
{
  #[inline(always)]
  fn get() -> Self {
    UnitValue::<Self>::get()
  }
}

impl<T> UnitType for T where T: Copy + Sized {}

#[derive(Copy, Clone)]
struct UnitValue<T>(PhantomData<T>)
where
  Self: Sized;

impl<T> UnitValue<T>
where
  Self: Copy + Sized,
{
  const SELF: Self = Self::new_checked();

  const fn new_checked() -> Self {
    // Statically assert that T is indeed a unit type.
    let size_must_be_0 = size_of::<T>();
    let s = Self(PhantomData::<T>);
    [s][size_must_be_0]
  }

  #[inline(always)]
  fn get_checked(self) -> T {
    // This run-time check serves just as a backup for the compile-time
    // check when Self::SELF is initialized.
    assert_eq!(size_of::<T>(), 0);
    unsafe { std::mem::MaybeUninit::<T>::zeroed().assume_init() }
  }

  #[inline(always)]
  pub fn get() -> T {
    // Accessing the Self::SELF is necessary to make the compile-time type check
    // work.
    Self::SELF.get_checked()
  }
}

pub trait MapFnFrom<F>
where
  Self: Sized,
{
  fn mapping() -> Self;

  #[inline(always)]
  fn map_fn_from(_: F) -> Self {
    Self::mapping()
  }
}

pub trait MapFnTo<T>
where
  Self: UnitType,
  T: Sized,
{
  fn mapping() -> T;

  #[inline(always)]
  fn map_fn_to(self) -> T {
    Self::mapping()
  }
}

impl<F, T> MapFnTo<T> for F
where
  Self: UnitType,
  T: MapFnFrom<F>,
{
  #[inline(always)]
  fn mapping() -> T {
    T::map_fn_from(F::get())
  }
}

pub trait CFnFrom<F>
where
  Self: Sized,
  F: UnitType,
{
  fn mapping() -> Self;

  #[inline(always)]
  fn c_fn_from(_: F) -> Self {
    Self::mapping()
  }
}

pub trait ToCFn<T>
where
  Self: UnitType,
  T: Sized,
{
  fn mapping() -> T;

  #[inline(always)]
  fn to_c_fn(self) -> T {
    Self::mapping()
  }
}

impl<F, T> ToCFn<T> for F
where
  Self: UnitType,
  T: CFnFrom<F>,
{
  #[inline(always)]
  fn mapping() -> T {
    T::c_fn_from(F::get())
  }
}

macro_rules! impl_c_fn_from {
  ($($arg:ident: $ty:ident),*) => {
    impl<F, R, $($ty),*> CFnFrom<F> for extern "C" fn($($ty),*) -> R
    where
      F: UnitType + Fn($($ty),*) -> R,
    {
      #[inline(always)]
      fn mapping() -> Self {
        extern "C" fn c_fn<F, R, $($ty),*>($($arg: $ty),*) -> R
        where
          F: UnitType + Fn($($ty),*) -> R,
        {
          (F::get())($($arg),*)
        }
        c_fn::<F, R, $($ty),*>
      }
    }
  };
}

impl_c_fn_from!();
impl_c_fn_from!(a0: A0);
impl_c_fn_from!(a0: A0, a1: A1);
impl_c_fn_from!(a0: A0, a1: A1, a2: A2);
impl_c_fn_from!(a0: A0, a1: A1, a2: A2, a3: A3);
impl_c_fn_from!(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4);
impl_c_fn_from!(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5);
impl_c_fn_from!(a0: A0, a1: A1, a2: A2, a3: A3, a4: A4, a5: A5, a6: A6);
