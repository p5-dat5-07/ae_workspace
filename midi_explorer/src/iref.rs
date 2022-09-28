std::sync::Arc;

struct IRef<B, R>
where B: 'static
{
    backing_value: Arc<B>,
    reference: R,
}

impl<B: 'static, R> IRef<B, R> {
    fn new(backing: B, builder: impl Fn(&B) -> R + '_) -> Self {
        let b = Arc::new(backing);
        let r = builder(&b);
        
        Self {
            backing_value: b,
            reference: r,
        }
    }
    
    fn try_new<E>(backing: B, builder: impl Fn(&B) -> Result<R, E>) -> Result<Self, E> {
        
    }
}

impl<B: 'static, R> std::ops::Deref for IRef<B, R> {
    type Target = R;
    
    fn deref(&self) -> &R {
        &self.ref
    }
}