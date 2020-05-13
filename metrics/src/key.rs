use crate::ScopedString;
use std::{fmt, slice::Iter};

/// A metric key.
///
/// A key always includes a name, but can optional include multiple labels used to further describe
/// the metric.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Key {
    name: ScopedString,
    labels: Option<Vec<Label>>,
}

/// A key/value pair used to further describe a metric.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct Label(pub(crate) ScopedString, pub(crate) ScopedString);

impl Key {
    /// Creates a `Key` from a name.
    pub fn from_name<N>(name: N) -> Self
    where
        N: Into<ScopedString>,
    {
        Key {
            name: name.into(),
            labels: None,
        }
    }

    /// Creates a `Key` from a name and vector of `Label`s.
    pub fn from_name_and_labels<N, L>(name: N, labels: L) -> Self
    where
        N: Into<ScopedString>,
        L: IntoLabels,
    {
        Key {
            name: name.into(),
            labels: Some(labels.into_labels()),
        }
    }

    /// Adds a new set of labels to this key.
    ///
    /// New labels will be appended to any existing labels.
    pub fn add_labels<L>(&mut self, new_labels: L)
    where
        L: IntoLabels,
    {
        let mut labels = self.labels.get_or_insert_with(|| Vec::new());
        labels.extend(new_labels.into_labels());
    }

    /// Name of this key.
    pub fn name(&self) -> ScopedString {
        self.name.clone()
    }

    /// Labels of this key, if they exist.
    pub fn labels(&self) -> Iter<Label> {
        match self.labels {
            Some(labels) => labels.iter(),
            None => [].iter(),
        }
    }

    /// Maps the name of this `Key` to a new name.
    pub fn map_name<F, S>(self, f: F) -> Self
    where
        F: FnOnce(ScopedString) -> S,
        S: Into<ScopedString>,
    {
        Key {
            name: f(self.name).into(),
            labels: self.labels,
        }
    }

    /// Consumes this `Key`, returning the name and any labels.
    pub fn into_parts(self) -> (ScopedString, Option<Vec<Label>>) {
        (self.name, self.labels)
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.labels.is_none() {
            write!(f, "Key({})", self.name)
        } else {
            let kv_pairs = self
                .labels
                .iter()
                .flatten()
                .map(|label| format!("{} = {}", label.0, label.1))
                .collect::<Vec<_>>();
            write!(f, "Key({}, [{}])", self.name, kv_pairs.join(", "))
        }
    }
}

impl From<String> for Key {
    fn from(name: String) -> Key {
        Key::from_name(name)
    }
}

impl From<&'static str> for Key {
    fn from(name: &'static str) -> Key {
        Key::from_name(name)
    }
}

impl From<ScopedString> for Key {
    fn from(name: ScopedString) -> Key {
        Key::from_name(name)
    }
}

impl<K, L> From<(K, L)> for Key
where
    K: Into<ScopedString>,
    L: IntoLabels,
{
    fn from(parts: (K, L)) -> Key {
        Key::from_name_and_labels(parts.0, parts.1)
    }
}

impl Label {
    /// Creates a `Label` from a key and value.
    pub fn new<K, V>(key: K, value: V) -> Self
    where
        K: Into<ScopedString>,
        V: Into<ScopedString>,
    {
        Label(key.into(), value.into())
    }

    /// The key of this label.
    pub fn key(&self) -> &str {
        self.0.as_ref()
    }

    /// The value of this label.
    pub fn value(&self) -> &str {
        self.1.as_ref()
    }

    /// Consumes this `Label`, returning the key and value.
    pub fn into_parts(self) -> (ScopedString, ScopedString) {
        (self.0, self.1)
    }
}

impl<K, V> From<(K, V)> for Label
where
    K: Into<ScopedString>,
    V: Into<ScopedString>,
{
    fn from(pair: (K, V)) -> Label {
        Label::new(pair.0, pair.1)
    }
}

impl<K, V> From<&(K, V)> for Label
where
    K: Into<ScopedString> + Clone,
    V: Into<ScopedString> + Clone,
{
    fn from(pair: &(K, V)) -> Label {
        Label::new(pair.0.clone(), pair.1.clone())
    }
}

/// A value that can be converted to `Label`s.
pub trait IntoLabels {
    /// Consumes this value, turning it into a vector of `Label`s.
    fn into_labels(self) -> Vec<Label>;
}

impl IntoLabels for Vec<Label> {
    fn into_labels(self) -> Vec<Label> {
        self
    }
}

impl<T, L> IntoLabels for &T
where
    Self: IntoIterator<Item = L>,
    L: Into<Label>,
{
    fn into_labels(self) -> Vec<Label> {
        self.into_iter().map(|l| l.into()).collect()
    }
}
