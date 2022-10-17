use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// An object that allows for an image type `I` to contain
/// references to heap allocated data of type `T` owned by the Lens.
/// 
/// Note: `I` being Rc/Arc is very very unsafe
pub struct Lens<T, I> {
    _data: T,
    image: I,
}

/// We implement constructors for boxed backing data types.
/// These constructors will move and box the given value.
impl<T, I> Lens<Box<T>, I>
where
    T: 'static,
{
    pub fn new(data: T, lens_fn: impl Fn(&'static T) -> I) -> Self {
        let boxed = Box::new(data);
        // safety: we guarantee at the API level that data is not mutated or moved
        //         and that the data and image lives equally as long.
        let image = lens_fn(unsafe { std::mem::transmute(boxed.as_ref()) });
        Self {
            _data: boxed,
            image,
        }
    }
    
    pub fn try_new<E>(data: T, lens_fn: impl Fn(&'static T) -> Result<I, E>) -> Result<Self, E> {
        // We reuse the `new_rc` implementation by creating a Lens with a Result as an image.
        let lens: Lens<Box<T>, Result<I, E>> = Lens::new(data, lens_fn);
        // Then we move the data and the actual image into a new Lens
        // with the desired type by mapping on the Result image.
        // Result<I, E> -> Result<Lens<T, I>, E>
        lens.image.map(move |image| {
            Self {
                _data: lens._data,
                image,
            }
        })
    }
}

// We implement constructors for reference counted backing data types
impl<T, I> Lens<T, I>
where
    T: 'static + ReferenceCounted,
{
    pub fn new_rc(data: T, lens_fn: impl Fn(&'static T::Inner) -> I) -> Self {
        // safety: we guarantee at the API level that data is not mutated or moved
        //         and that the data and image lives equally as long.
        let image = lens_fn(unsafe { std::mem::transmute(data.as_ref()) });
        Self {
            _data: data,
            image,
        }
    }
    
    pub fn try_new_rc<E>(data: T, lens_fn: impl Fn(&'static T::Inner) -> Result<I, E>) -> Result<Self, E> {
        // We reuse the `new_rc` implementation by creating a Lens with a Result as an image.
        let lens: Lens<T, Result<I, E>> = Lens::new_rc(data, lens_fn);
        // Then we move the data and the actual image into a new Lens
        // with the desired type by mapping on the Result image.
        // Result<I, E> -> Result<Lens<T, I>, E>
        lens.image.map(move |image| {
            Self {
                _data: lens._data,
                image,
            }
        })
    }
    
    pub fn reimage<R>(&self, lens_fn: impl Fn(&'static T::Inner) -> R) -> Lens<T, R> {
        Lens::new_rc(self._data.clone(), lens_fn)
    }
    
    pub fn clone_map<U>(&self, map: impl Fn(&I) -> U) -> Lens<T, U> {
        let data = self._data.clone();
        Lens {
            _data: data,
            image: map(&self.image),
        }
    }
}

// Allows us to use the lens as if it was the image object
impl<T, I> Deref for Lens<T, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}

impl<T, I> DerefMut for Lens<T, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.image   
    }
}

pub trait ReferenceCounted: Sized + Clone + AsRef<Self::Inner> {
    type Inner;

    fn new(v: Self::Inner) -> Self;
}

impl<T> ReferenceCounted for Rc<T> {
    type Inner = T;

    fn new(v: T) -> Self {
        Rc::new(v)
    }
}

impl<T> ReferenceCounted for std::sync::Arc<T> {
    type Inner = T;

    fn new(v: T) -> Self {
        std::sync::Arc::new(v)
    }
}
