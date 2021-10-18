use crate::vanilla::namespace::Namespace;

pub struct Scope<'a> {
    pub name: String,

    pub namespace: &'a Namespace<'a>,
    pub parent: Option<Box<Scope<'a>>>,

    pub content: String,
}

impl<'a> Scope<'a> {
    pub fn get_reference_name(&self) -> String {
        format!("{}:{}", &self.namespace.name, &self.name)
    }

    pub fn new_unnamed(id: usize, namespace: &'a Namespace<'a>) -> Self {
        let name = format!("_/{:02X}/{:02X}/{:02X}/{:02X}",
            (id / ( 1 << 24)) & 255,
            (id / ( 1 << 16)) & 255,
            (id / ( 1 << 8)) & 255,
            (id / ( 1 << 0)) & 255,
        );

        Self { name, namespace, parent: None, content: String::new() }
    }

    pub fn set_reference_id(&mut self, id: usize) {
        self.name = format!("_/{:02X}/{:02X}/{:02X}/{:02X}",
            (id / ( 1 << 24)) & 255,
            (id / ( 1 << 16)) & 255,
            (id / ( 1 << 8)) & 255,
            (id / ( 1 << 0)) & 255,
        );
    }

    pub fn new(name: String, namespace: &'a Namespace<'a>) -> Self {
        Self { name, namespace, parent: None, content: String::new() }
    }
}
