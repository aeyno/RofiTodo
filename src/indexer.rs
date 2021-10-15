use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;
use std::cmp::Ordering;

type CompareFunction<T> = fn(&T,&T) -> Ordering;

/// Store data data with mutiple indexes and filters
pub struct Indexer<T> {
    /// An index which stores all the values without filtering
    main_index : BTreeSet<Rc<T>>,
    /// An hashmap of indexes for storing all the filtered and ordered data
    indexes : HashMap<String, Index<T>>
}

impl<T : std::cmp::Ord> Indexer<T> {

    /// Create new Indexer
    pub fn new() -> Self {
        Indexer { main_index : BTreeSet::<Rc<T>>::new() , indexes : HashMap::<String,Index<T>>::new() }
    }

    /// Index a new element into the structure
    /// 
    /// Arguments:
    /// 
    /// * `element` - the element to index
    pub fn add(&mut self, element : T) -> Rc<T> {
        let e = Rc::new(element);
        self.main_index.insert(Rc::clone(&e));
        for (_, index) in &mut self.indexes {
            index.register(Rc::clone(&e));
        }
        e
    }

    /// Remove an element from the structure
    /// 
    /// Arguments:
    /// 
    /// * `element` - Rc reference to the element to remove
    pub fn remove(&mut self, element : Rc<T>) -> Option<T> {
        self.main_index.remove(&element);
        let mut empty_indexes = Vec::<String>::new();
        for (name, index) in &mut self.indexes {
            index.remove(&element);
            if index.remove_if_empty() && index.is_empty() {
                empty_indexes.push(name.to_string());
            }
        }
        for name in empty_indexes {
            self.remove_index(&name);
        }
        match Rc::try_unwrap(element) {
            Ok(task) => Some(task),
            _ => None
        }
    }

    /// Get the main index to iterate over all the stored elements
    pub fn get_main_index(&self) -> &BTreeSet<Rc<T>> {
        &self.main_index
    }

    /// Create a new index for the data
    /// Sort all the existing data at the creation of the index
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the new index
    /// * `filter` - a closure to filter the elements (returns `true` if the value should be in the index)
    /// * `compare_fn` - a function to compare and sort elements
    pub fn new_index(&mut self, name : String, filter : impl Fn(&T) -> bool + 'static, compare_fn : CompareFunction<T>) {
        match self.get_index(&name) {
            None => {
                let mut new_idx = Index::new(filter, compare_fn);
                for x in &self.main_index {
                    new_idx.register(Rc::clone(&x));
                }
                self.indexes.insert(name.clone(), new_idx);
            },
            _ => ()
        }
    }

    /// Create a new index for the data which is automatically removed when empty
    /// Sort all the existing data at the creation of the index
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the new index
    /// * `filter` - a closure to filter the elements (returns `true` if the value should be in the index)
    pub fn new_autoremove_index(&mut self, name : String, filter : impl Fn(&T) -> bool + 'static, compare_fn : CompareFunction<T>) {
        match self.get_index(&name) {
            None => {
                let mut new_idx = Index::new_autoremove(filter, compare_fn);
                for x in &self.main_index {
                    new_idx.register(Rc::clone(&x));
                }
                self.indexes.insert(name.clone(), new_idx);
            },
            _ => ()
        }
    }

    /// Remove an index
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the index
    pub fn remove_index(&mut self, name: &String) -> () {
        self.indexes.remove(name);
    }

    /// Return a BTreeSet with the content of an Index
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the index
    pub fn get_index(&self, name : &String) -> Option<&BTreeSet<ElementWrapper<T>>> {
        match self.indexes.get(name) {
            Some(i) => Some(&i.content),
            None => None
        }
    }

    /// Return an reference to an Index
    /// 
    /// Arguments:
    /// 
    /// * `name` - the name of the index
    pub fn index(&self, name : &String) -> Option<&Index<T>> {
        self.indexes.get(name)
    }
}

/// Contains a sorted list of pointers to elements
pub struct Index<T> {
    /// The content of the Index
    content : BTreeSet<ElementWrapper<T>>,
    /// A closure to filter the elements (returns `true` if the value should be in the index)
    is_indexable : Box<dyn Fn(&T) -> bool>,
    /// A function to compare and sort elements
    compare : CompareFunction<T>,
    /// Indicates whether the Index should be removed when empty
    remove_if_empty : bool
}

impl<T> Index<T> {
    /// Create a new Index
    /// the `is_indexable` function is used to filter the elements
    /// the `compare` function is used to sort the task inside the index
    /// 
    /// Arguments:
    /// 
    /// * `is_indexable` - a closure to filter the elements (returns `true` if the value should be in the index)
    /// * `compare_fn` - a function to compare and sort elements
    pub fn new(is_indexable : impl Fn(&T) -> bool + 'static, compare : CompareFunction<T>) -> Self {
        Index { content : BTreeSet::<ElementWrapper<T>>::new(), is_indexable : Box::new(is_indexable), compare : compare , remove_if_empty : false }
    }

    /// Create a new Index which is removed when empty
    /// the `is_indexable` function is used to filter the elements
    /// the `compare` function is used to sort the task inside the index
    /// 
    /// Arguments:
    /// 
    /// * `is_indexable` - a closure to filter the elements (returns `true` if the value should be in the index)
    /// * `compare_fn` - a function to compare and sort elements
    pub fn new_autoremove(is_indexable : impl Fn(&T) -> bool + 'static, compare : CompareFunction<T>) -> Self {
        Index { content : BTreeSet::<ElementWrapper<T>>::new(), is_indexable : Box::new(is_indexable), compare : compare , remove_if_empty : true }
    }

    /// Register a new element in the Index
    /// 
    /// Arguments:
    /// 
    /// * `element` - a boxed element
    pub fn register(&mut self, element : Rc<T>) -> () {
        if (self.is_indexable)(element.as_ref()) {
            let ew = ElementWrapper::new(element, self.compare);
            self.content.insert(ew);
        }
    }

    /// Remove an element from the Index
    /// 
    /// Arguments:
    /// 
    /// * `element` - a reference to boxed element
    pub fn remove(&mut self, element : &Rc<T>) {
        if (self.is_indexable)(&element) {
            self.content.remove(&ElementWrapper::new(Rc::clone(element), self.compare));
        }
    }

    /// Return true if the Index is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Return true if the Index should be removed when empty
    pub fn remove_if_empty(&self) -> bool {
        self.remove_if_empty
    }
}

/// Create an object to iterate over an Index without consuming it's content
impl<'a, T> IntoIterator for &'a Index<T> {
    type Item = Rc<T>;
    type IntoIter = IndexIntoIterator<'a,T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            btree_ite : Box::new(self.content.iter())
        }
    }
}

/// A struct to iterate over an Index without consuming it's content
pub struct IndexIntoIterator<'a, T> {
    btree_ite : Box<std::collections::btree_set::Iter<'a, ElementWrapper<T>>>,
}

/// Implements the ability to iterate over IndexIntoIterator
impl<'a,T> Iterator for IndexIntoIterator<'a, T> {
    type Item = Rc<T>;

    /// Get the next element of the iterator
    fn next(&mut self) -> Option<Self::Item> {
        match self.btree_ite.next() {
            Some(elem) => Some(Rc::clone(&elem.content)),
            None => None
        }
    }
}

/// An element of the Index which wrap the element to allow sorting
pub struct ElementWrapper<T> {
    /// A smart pointer to the element
    content : Rc<T>,
    /// A function to compare two elements
    compare : CompareFunction<T>
}

impl<T> ElementWrapper<T> {
    /// Create a new ElementWrapper for an element
    /// 
    /// Arguments:
    /// 
    /// * `elem` - a smart pointer to an element
    /// * `compare_fn` - a function to compare and sort elements
    fn new(elem : Rc<T>, compare_fn : CompareFunction<T>) -> Self {
        ElementWrapper { content : elem, compare : compare_fn }
    }

    /// Return a reference to the content
    pub fn _content(&self) -> &T {
        self.content.as_ref()
    }
}

/// Implementing ordering for `ElementWrapper` to allow BTreeSet to sort it
impl<T> Ord for ElementWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.compare)(self.content.as_ref(), other.content.as_ref())
    }
}

/// Implementing `PartialOrd` to implement `Ord` for `ElementWrapper`
impl<T> PartialOrd for ElementWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.compare)(self.content.as_ref(), other.content.as_ref()))
    }
}

/// Implementing `PartialEq` to implement `PartialOrd` for `ElementWrapper`
impl<T> PartialEq for ElementWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.content, &other.content) { return true }
        (self.compare)(self.content.as_ref(), other.content.as_ref()) == Ordering::Equal
    }
}

/// Implementing `Eq` to implement `PartialEq` for `ElementWrapper`
impl<T> Eq for ElementWrapper<T> { }

#[cfg(test)]
mod index_tests {
    use super::*;

    #[test]
    fn create_index() {
        let idx = Index::<String>::new(|_|true, String::cmp);
        assert_eq!(idx.is_empty(), true);
        assert_eq!(idx.remove_if_empty(), false);
    }
    
    #[test]
    fn create_remove_if_empty_index() {
        let idx = Index::<String>::new_autoremove(|_|true, String::cmp);
        assert_eq!(idx.is_empty(), true);
        assert_eq!(idx.remove_if_empty(), true);
    }

    #[test]
    fn indexing_elements() {
        let mut idx = Index::<String>::new_autoremove(|_|true, String::cmp);
        let foo = Rc::new(String::from("Foo"));
        let bar = Rc::new(String::from("Bar"));
        let baz = Rc::new(String::from("Baz"));
        idx.register(Rc::clone(&foo));
        idx.register(Rc::clone(&bar));
        idx.register(Rc::clone(&baz));

        let data = idx.into_iter().collect::<Vec<_>>();
        assert_eq!(data[0], bar);
        assert_eq!(data[1], baz);
        assert_eq!(data[2], foo);
        assert_eq!(idx.is_empty(), false);
    }

    #[test]
    fn iter_dont_consume() {
        let mut idx = Index::<String>::new_autoremove(|_|true, String::cmp);
        let foo = Rc::new(String::from("Foo"));
        let bar = Rc::new(String::from("Bar"));
        let baz = Rc::new(String::from("Baz"));
        idx.register(Rc::clone(&foo));
        idx.register(Rc::clone(&bar));
        idx.register(Rc::clone(&baz));

        // We can iterate twice on the elements without consuming the Index
        let _data1 = idx.into_iter().collect::<Vec<_>>();
        let _data2 = idx.into_iter().collect::<Vec<_>>();
    }

    #[test]
    fn remove_elements() {
        let mut idx = Index::<String>::new_autoremove(|_|true, String::cmp);
        let foo = Rc::new(String::from("Foo"));
        let bar = Rc::new(String::from("Bar"));
        let baz = Rc::new(String::from("Baz"));
        idx.register(Rc::clone(&foo));
        idx.register(Rc::clone(&bar));
        idx.register(Rc::clone(&baz));

        let data = idx.into_iter().collect::<Vec<_>>();
        
        assert!(!idx.is_empty());

        idx.remove(&data[0]);
        idx.remove(&data[2]);
        idx.remove(&data[1]);

        assert!(idx.is_empty());
    }
}

#[cfg(test)]
mod indexer_tests {
    use super::*;

    fn filt(_s : &String) -> bool {
        true
    }

    fn lexicographic(s1 : &String, s2 : &String) -> Ordering {
        if s1<s2 {
            Ordering::Less
        } else if s1 == s2 {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    }

    fn lexicographic_reverse(s1 : &String, s2 : &String) -> Ordering {
        if s1<s2 {
            Ordering::Greater
        } else if s1 == s2 {
            Ordering::Equal
        } else {
            Ordering::Less
        }
    }

    #[test]
    fn create_indexer() {
        let idxname = String::from("Alpha");
        let mut id = Indexer::<String>::new();
        id.new_index(idxname.clone(), filt, lexicographic);
        id.add(String::from("foo"));
        id.add(String::from("bar"));

        let mut iter = id.get_index(&idxname).unwrap().iter();
        assert_eq!(iter.next().unwrap()._content(), &String::from("bar"));
        assert_eq!(iter.next().unwrap()._content(), &String::from("foo"));
    }

    #[test]
    fn create_multiple_indexes() {
        let idxname1 = String::from("Alpha");
        let idxname2 = String::from("AlphaRev");
        let mut id = Indexer::<String>::new();
        id.new_index(idxname1.clone(), filt, lexicographic);
        id.add(String::from("foo"));
        id.add(String::from("bar"));
        id.add(String::from("hello"));

        id.new_index(idxname2.clone(), filt, lexicographic_reverse);

        let lexico = id.index(&idxname1).unwrap().into_iter().collect::<Vec<_>>();
        let lexicorev = id.index(&idxname2).unwrap().into_iter().collect::<Vec<_>>();

        let l = lexico.len();
        for i in 0..l {
            assert_eq!(lexico[i], lexicorev[l-1-i]);
        }
    }

    #[test]
    fn remove_indexes() {
        let idxname1 = String::from("Alpha");
        let idxname2 = String::from("AlphaRev");
        let mut id = Indexer::<String>::new();
        id.new_index(idxname1.clone(), filt, lexicographic);
        id.new_index(idxname2.clone(), filt, lexicographic_reverse);
        id.add(String::from("foo"));
        id.add(String::from("bar"));
        id.add(String::from("hello"));

        id.remove_index(&idxname1);
        assert!(id.index(&idxname1).is_none());
        assert!(!id.index(&idxname2).is_none());
        assert!(!id.index(&idxname2).unwrap().is_empty());
    }

    #[test]
    fn auto_remove_indexes() {
        let idxname1 = String::from("Alpha");
        let idxname2 = String::from("AlphaRev");
        let mut id = Indexer::<String>::new();
        id.new_autoremove_index(idxname1.clone(), filt, lexicographic);
        id.new_index(idxname2.clone(), filt, lexicographic);
        id.add(String::from("foo"));
        id.add(String::from("bar"));
        id.add(String::from("baz"));

        assert!(!id.index(&idxname1).is_none());
        assert!(!id.index(&idxname2).is_none());

        let elems = id.index(&idxname1).unwrap().into_iter().collect::<Vec<_>>();
        for elem in elems {
            id.remove(elem);
        }

        assert!(id.index(&idxname1).is_none());
        assert!(!id.index(&idxname2).is_none());
    }

    #[test]
    fn testing_iterator() {
        let idxname = String::from("Alpha");
        let mut id = Indexer::<String>::new();
        id.new_index(idxname.clone(), filt, lexicographic);
        id.add(String::from("foo"));
        id.add(String::from("bar"));
        id.add(String::from("foobar"));

        let mut iter = id.get_index(&idxname).unwrap().iter();
        for x in id.index(&idxname).unwrap() {
            assert_eq!(iter.next().unwrap()._content(), x.as_ref());
        }
    }

    #[test]
    fn testing_remove() {
        let idxname1 = String::from("Alpha");
        let idxname2 = String::from("AlphaRev");
        let mut id = Indexer::<String>::new();
        id.new_index(idxname1.clone(), filt, lexicographic);
        id.new_index(idxname2.clone(), filt, lexicographic_reverse);
        id.add(String::from("foo"));
        id.add(String::from("bar"));
        id.add(String::from("hello"));

        let item = id.index(&idxname1).unwrap().into_iter().next().unwrap();

        id.remove(Rc::clone(&item));

        let l1 = id.index(&idxname1).unwrap().into_iter().collect::<Vec<_>>();
        let l2 = id.index(&idxname1).unwrap().into_iter().collect::<Vec<_>>();
        let l3 = id.get_main_index().into_iter().collect::<Vec<_>>();
        assert_eq!(l1.len(), 2);
        assert_eq!(l2.len(), 2);
        assert_eq!(l3.len(), 2);
    }
}