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

impl<T> From<Vec2<T>> for [T; 2]
where
    T: Copy,
{
    fn from(vec2: Vec2<T>) -> Self {
        [vec2.x, vec2.y]
    }
}