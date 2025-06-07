pub trait LoadSerde {
    type SerdeData: serde::Serialize;
    fn load_serde(&self) -> &Self::SerdeData;
}

impl<T> LoadSerde for T
where
    T: serde::Serialize + 'static,
{
    type SerdeData = Self;

    fn load_serde(&self) -> &Self::SerdeData { self }
}
