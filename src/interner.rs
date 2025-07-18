use gxhash::{HashMap, HashMapExt};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Id(usize);

impl Id {
    /// # Safety
    /// Id's should not be manually created
    #[inline(always)]
    pub unsafe fn from_usize(value: usize) -> Id {
        Id(value)
    }
}

#[derive(Default)]
pub struct Interner {
    // This can be static as index will always be dropped before data is dropped
    index: HashMap<&'static str, Id>,
    data: Vec<Box<str>>,
}

impl Interner {
    pub fn new() -> Interner {
        Interner {
            index: HashMap::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, string: &str) -> Id {
        if let Some(id) = self.index.get(string) {
            return *id;
        }

        let string = Box::<str>::from(string);

        let static_ref = unsafe { &*((&*string) as *const str) };

        self.data.push(string);

        let id = Id(self.data.len() - 1);
        self.index.insert(static_ref, id);

        id
    }

    #[inline(always)]
    pub fn resolve(&self, id: Id) -> &str {
        &self.data[id.0]
    }

    /// # Safety
    /// `id` must be less than `self.data.len()`
    #[inline(always)]
    pub fn resolve_unchecked(&self, id: Id) -> &str {
        unsafe { self.data.get_unchecked(id.0) }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_access() {
        let mut interner = Interner::new();

        let a = interner.insert("a");
        let b = interner.insert("b");

        assert_eq!(interner.resolve(a), "a");
        assert_eq!(interner.resolve(b), "b");
    }

    #[test]
    fn correct_access_unchecked() {
        let mut interner = Interner::new();

        let a = interner.insert("a");
        let b = interner.insert("b");

        assert_eq!(interner.resolve_unchecked(a), "a");
        assert_eq!(interner.resolve_unchecked(b), "b");
    }

    #[test]
    fn correct_len() {
        let mut interner = Interner::new();

        let _a = interner.insert("a");
        let _b = interner.insert("b");

        assert_eq!(interner.len(), 2);
        assert!(!interner.is_empty());
    }

    #[test]
    fn correct_is_empty() {
        let interner = Interner::new();

        assert!(interner.is_empty());
    }

    #[test]
    fn double_insert() {
        let mut interner = Interner::new();

        let a = interner.insert("hello");
        let b = interner.insert("hello");

        assert_eq!(a, b);

        assert_eq!(interner.len(), 1);

        assert_eq!(interner.resolve(a), "hello");
        assert_eq!(interner.resolve(b), "hello");
    }

    #[test]
    fn correct_reference_behaviour() {
        let mut interner = Interner::new();

        let mut string = String::from("hello");

        let a = interner.insert(&string);

        string.clear();

        assert_eq!(interner.resolve(a), "hello");
    }
}
