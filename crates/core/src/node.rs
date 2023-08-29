// TODO: include graph

pub struct Node<'a, N, W> {
    id: &'a N,

    weight: &'a W,
}

impl<'a, N, W> Node<'a, N, W> {
    pub fn new(id: &'a N, weight: &'a W) -> Self {
        Self { id, weight }
    }

    pub fn id(&self) -> &'a N {
        self.id
    }

    pub fn weight(&self) -> &'a W {
        self.weight
    }
}

pub struct NodeMut<'a, N, W> {
    id: &'a N,

    weight: &'a mut W,
}

impl<'a, N, W> NodeMut<'a, N, W> {
    pub fn id(&self) -> &'a N {
        self.id
    }

    pub fn weight(&self) -> &'a W {
        self.weight
    }

    pub fn weight_mut(&mut self) -> &'a mut W {
        self.weight
    }
}

// TODO: consider naming this `FreeNode`
pub struct DetachedNode<N, W> {
    pub id: N,

    pub weight: W,
}

impl<N, W> DetachedNode<N, W> {
    pub fn new(id: N, weight: W) -> Self {
        Self { id, weight }
    }
}
