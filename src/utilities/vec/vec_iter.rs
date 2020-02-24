use super::Axis;

pub trait GameVec<T: Copy> {
    fn x(&self) -> &T;
    fn y(&self) -> &T;
}

pub struct VecIter<'a, T: Copy> {
    inner: &'a dyn GameVec<T>,
    axis: Option<Axis>,
}

impl<'a, T: Copy> VecIter<'a, T> {
    pub fn new(inner: &'a dyn GameVec<T>) -> VecIter<'a, T> {
        VecIter {
            inner,
            axis: Some(Axis::X),
        }
    }
}

impl<'a, T: Copy> Iterator for VecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.axis.map(|axis| match axis {
            Axis::X => {
                self.axis = Some(Axis::Y);
                self.inner.x()
            }

            Axis::Y => {
                self.axis = None;
                self.inner.y()
            }
        })
    }
}
