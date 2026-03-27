#[derive(Debug, PartialEq)]
pub struct Relation {
    pub field:            String,
    pub references:       String,
    pub references_field: String,
}

#[derive(Debug, PartialEq)]
pub struct InverseRelation {
    pub table_name:  String,
    pub entity_name: String,
    pub fk_field:    String,
    pub local_field: String,
}
