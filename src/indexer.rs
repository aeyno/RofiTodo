use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;
use std::cmp::Ordering;

type CompareFunction<T> = fn(&T,&T) -> Ordering;

pub struct Indexer<T> {
    /// An index which stores all the values without filtering
    main_index : BTreeSet<Rc<T>>,
    /// An hashmap of indexes for storing all the filtered and ordered data
    indexes : HashMap<String, Index<T>>
}

impl<T : std::cmp::Ord> Indexer<T> {
    pub fn new() -> Self {
        Indexer { main_index : BTreeSet::<Rc<T>>::new() , indexes : HashMap::<String,Index<T>>::new() }
    }

    pub fn add(&mut self, element : T) -> Rc<T> {
        let e = Rc::new(element);
        self.main_index.insert(Rc::clone(&e));
        for (_, index) in &mut self.indexes {
            index.register(Rc::clone(&e));
        }
        e
    }

    pub fn get_main_index(&self) -> &BTreeSet<Rc<T>> {
        &self.main_index
    }

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

    pub fn remove_index(&mut self, name: &String) -> () {
        self.indexes.remove(name);
    }

    pub fn get_index(&self, name : &String) -> Option<&BTreeSet<ElementWrapper<T>>> {
        match self.indexes.get(name) {
            Some(i) => Some(&i.content),
            None => None
        }
    }

    pub fn index(&self, name : &String) -> Option<&Index<T>> {
        self.indexes.get(name)
    }
}

pub struct Index<T> {
    content : BTreeSet<ElementWrapper<T>>,
    is_indexable :  Box<dyn Fn(&T) -> bool>,
    compare : CompareFunction<T>,
    remove_if_empty : bool
}

impl<T> Index<T> {
    pub fn new(is_indexable : impl Fn(&T) -> bool + 'static, compare : CompareFunction<T>) -> Self {
        Index { content : BTreeSet::<ElementWrapper<T>>::new(), is_indexable : Box::new(is_indexable), compare : compare , remove_if_empty : false }
    }

    pub fn new_autoremove(is_indexable : impl Fn(&T) -> bool + 'static, compare : CompareFunction<T>) -> Self {
        Index { content : BTreeSet::<ElementWrapper<T>>::new(), is_indexable : Box::new(is_indexable), compare : compare , remove_if_empty : true }
    }

    pub fn register(&mut self, element : Rc<T>) -> () {
        if (self.is_indexable)(element.as_ref()) {
            let ew = ElementWrapper::new(element, self.compare);
            self.content.insert(ew);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn remove_if_empty(&self) -> bool {
        self.remove_if_empty
    }

    pub fn remove(&mut self, element : &Rc<T>) {
        if (self.is_indexable)(&element) {
            self.content.remove(&ElementWrapper::new(Rc::clone(element), self.compare));
        }
    }
}

impl<'a, T> IntoIterator for &'a Index<T> {
    type Item = Rc<T>;
    type IntoIter = IndexIntoIterator<'a,T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            btree_ite : Box::new(self.content.iter())
        }
    }
}

pub struct IndexIntoIterator<'a, T> {
    btree_ite : Box<std::collections::btree_set::Iter<'a, ElementWrapper<T>>>,
}

impl<'a,T> Iterator for IndexIntoIterator<'a, T> {
    type Item = Rc<T>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.btree_ite.next() {
            Some(elem) => Some(Rc::clone(&elem.content)),
            None => None
        }
    }
}

pub struct ElementWrapper<T> {
    content : Rc<T>,
    compare : CompareFunction<T>
}

impl<T> ElementWrapper<T> {
    fn new(elem : Rc<T>, compare_fn : CompareFunction<T>) -> Self {
        ElementWrapper { content : elem, compare : compare_fn }
    }

    pub fn _content(&self) -> &T {
        self.content.as_ref()
    }
}

impl<T> Ord for ElementWrapper<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.compare)(self.content.as_ref(), other.content.as_ref())
    }
}

impl<T> PartialOrd for ElementWrapper<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some((self.compare)(self.content.as_ref(), other.content.as_ref()))
    }
}

impl<T> PartialEq for ElementWrapper<T> {
    fn eq(&self, other: &Self) -> bool {
        if Rc::ptr_eq(&self.content, &other.content) { return true }
        (self.compare)(self.content.as_ref(), other.content.as_ref()) == Ordering::Equal
    }
}

impl<T> Eq for ElementWrapper<T> { }

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