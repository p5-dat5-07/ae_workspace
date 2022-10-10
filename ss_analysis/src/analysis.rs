struct Matrix(Vec<f64>);

impl Matrix {
    fn new(size: usize) -> Self {
        let n = (size * (size + 1)) / 2;
        Self(Vec::with_capacity(n))
    }
}

impl<T: Copy + Into<usize>> std::ops::Index<(T, T)> for Matrix {
    type Output = f64;

    fn index(&self, index: (T, T)) -> &Self::Output {
        let i: (usize, usize) = (index.0.into(), index.1.into());
        let (x, y) = match i {
            (x, y) if x > y => (y, x),
            (x, y) if x == y => return &1.0,
            _ => i,
        };

        &1.0
    }
}
