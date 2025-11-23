use std::ffi::CStr;

pub fn compare_slices<V1>(v1: &[i8], v2: &[i8]) -> bool {
    v1.cmp(v2).is_eq()
}
pub trait IteratorTryAny<T, E>: Iterator<Item = Result<T, E>> + Sized {
    fn try_any<F>(self, mut pred: F) -> Result<bool, E>
    where
        F: FnMut(&T) -> bool,
    {
        for item in self {
            let value = item?;
            if pred(&value) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl<I, T, E> IteratorTryAny<T, E> for I where I: Iterator<Item = Result<T, E>> {}

pub fn slice_cstr_to_ptr(input: &[&CStr]) -> Vec<*const i8> {
    input.iter().map(|s| s.as_ptr()).collect::<Vec<_>>()
}
