#[derive(Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Copy for Vec2<T> where T: Copy {}

impl<T> Clone for Vec2<T>
where
    T: Copy,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Vec2<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> From<[T; 2]> for Vec2<T>
where
    T: Copy,
{
    fn from(array: [T; 2]) -> Self {
        Self {
            x: array[0],
            y: array[1],
        }
    }
}

impl<T> Into<[T; 2]> for Vec2<T>
where
    T: Copy,
{
    fn into(self) -> [T; 2] {
        [self.x, self.y]
    }
}
