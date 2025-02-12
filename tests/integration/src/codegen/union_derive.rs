//! Tests for `#[derive(GraphQLUnion)]` macro.

use std::marker::PhantomData;

use juniper::{
    execute, graphql_object, graphql_value, graphql_vars, DefaultScalarValue, EmptyMutation,
    EmptySubscription, GraphQLObject, GraphQLType, GraphQLUnion, RootNode, ScalarValue,
};

#[derive(GraphQLObject)]
struct Human {
    id: String,
    home_planet: String,
}

#[derive(GraphQLObject)]
struct Droid {
    id: String,
    primary_function: String,
}

#[derive(GraphQLObject)]
struct Ewok {
    id: String,
    funny: bool,
}

pub enum CustomContext {
    Human,
    Droid,
    Ewok,
}
impl juniper::Context for CustomContext {}

#[derive(GraphQLObject)]
#[graphql(context = CustomContext)]
pub struct HumanCustomContext {
    id: String,
    home_planet: String,
}

#[derive(GraphQLObject)]
#[graphql(context = CustomContext)]
pub struct DroidCustomContext {
    id: String,
    primary_function: String,
}

#[derive(GraphQLObject)]
#[graphql(context = CustomContext)]
struct EwokCustomContext {
    id: String,
    funny: bool,
}

fn schema<'q, C, Q>(query_root: Q) -> RootNode<'q, Q, EmptyMutation<C>, EmptySubscription<C>>
where
    Q: GraphQLType<DefaultScalarValue, Context = C, TypeInfo = ()> + 'q,
{
    RootNode::new(
        query_root,
        EmptyMutation::<C>::new(),
        EmptySubscription::<C>::new(),
    )
}

fn schema_with_scalar<'q, S, C, Q>(
    query_root: Q,
) -> RootNode<'q, Q, EmptyMutation<C>, EmptySubscription<C>, S>
where
    Q: GraphQLType<S, Context = C, TypeInfo = ()> + 'q,
    S: ScalarValue + 'q,
{
    RootNode::new_with_scalar_value(
        query_root,
        EmptyMutation::<C>::new(),
        EmptySubscription::<C>::new(),
    )
}

mod trivial_enum {
    use super::*;

    #[derive(GraphQLUnion)]
    enum Character {
        A(Human),
        B(Droid),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn is_graphql_union() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "UNION"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn uses_type_name() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                name
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"name": "Character"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                description
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod generic_enum {
    use super::*;

    #[derive(GraphQLUnion)]
    enum Character<A, B> {
        A(Human),
        B(Droid),
        #[graphql(ignore)]
        _State(A, B),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character<u8, ()> {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_type_name_without_type_params() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                name
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"name": "Character"}}), vec![])),
        );
    }
}

/* TODO: make it work
mod generic_lifetime_enum {
    use super::*;

    #[derive(GraphQLObject)]
    struct LifetimeHuman<'id> {
        id: &'id str,
    }

    #[derive(GraphQLObject)]
    struct GenericDroid<B = ()> {
        id: String,
        #[graphql(ignore)]
        _t: PhantomData<B>,
    }

    #[derive(GraphQLUnion)]
    enum Character<'id, B = ()> {
        A(LifetimeHuman<'id>),
        B(GenericDroid<B>),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character<'_> {
            match self {
                Self::Human => Character::A(LifetimeHuman { id: "human-32" }),
                Self::Droid => Character::B(GenericDroid {
                    id: "droid-99".into(),
                    _t: PhantomData,
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on LifetimeHuman {
                humanId: id
            }
            ... on GenericDroid {
                droidId: id
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_type_name_without_type_params() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                name
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"name": "Character"}}), vec![])),
        );
    }
}
*/

mod description_from_doc_comments {
    use super::*;

    /// Rust docs.
    #[derive(GraphQLUnion)]
    enum Character {
        A(Human),
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            Character::A(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            })
        }
    }

    #[tokio::test]
    async fn resolves_human() {
        const DOC: &str = r#"{
            character {
                ... on Human {
                    humanId: id
                    homePlanet
                }
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_doc_comment_as_description() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Rust docs."}}),
                vec![],
            )),
        );
    }
}

mod explicit_name_and_description {
    use super::*;

    /// Rust docs.
    #[derive(GraphQLUnion)]
    #[graphql(name = "MyChar", desc = "My character.")]
    enum Character {
        A(Human),
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            Character::A(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            })
        }
    }

    #[tokio::test]
    async fn resolves_human() {
        const DOC: &str = r#"{
            character {
                ... on Human {
                    humanId: id
                    homePlanet
                }
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_custom_name() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                name
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"name": "MyChar"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn uses_custom_description() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "My character."}}),
                vec![],
            )),
        );
    }
}

mod explicit_scalar {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(scalar = DefaultScalarValue)]
    enum Character {
        A(Human),
        B(Droid),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod custom_scalar {
    use crate::custom_scalar::MyScalarValue;

    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(scalar = MyScalarValue)]
    enum Character {
        A(Human),
        B(Droid),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(scalar = MyScalarValue)]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod explicit_generic_scalar {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(scalar = S)]
    enum Character<S: ScalarValue> {
        A(Human),
        B(Droid),
        #[graphql(ignore)]
        _P(PhantomData<S>),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character<__S: ScalarValue>(&self) -> Character<__S> {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod bounded_generic_scalar {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(scalar = S: ScalarValue + Clone)]
    enum Character {
        A(Human),
        B(Droid),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod custom_context {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = CustomContext)]
    enum Character {
        A(HumanCustomContext),
        B(DroidCustomContext),
    }

    struct QueryRoot;

    #[graphql_object(context = CustomContext)]
    impl QueryRoot {
        fn character(&self, ctx: &CustomContext) -> Character {
            match ctx {
                CustomContext::Human => Character::A(HumanCustomContext {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                CustomContext::Droid => Character::B(DroidCustomContext {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
                _ => unimplemented!(),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on HumanCustomContext {
                humanId: id
                homePlanet
            }
            ... on DroidCustomContext {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Human).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Droid).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod different_context {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = CustomContext)]
    enum Character {
        A(HumanCustomContext),
        B(Droid),
    }

    struct QueryRoot;

    #[graphql_object(context = CustomContext)]
    impl QueryRoot {
        fn character(&self, ctx: &CustomContext) -> Character {
            match ctx {
                CustomContext::Human => Character::A(HumanCustomContext {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                CustomContext::Droid => Character::B(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
                _ => unimplemented!(),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on HumanCustomContext {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Human).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Droid).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod ignored_enum_variants {
    use super::*;

    #[derive(GraphQLUnion)]
    enum Character {
        A(Human),
        #[graphql(ignore)]
        _C(Ewok),
        #[graphql(skip)]
        _D,
    }

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            Character::A(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            })
        }
    }

    #[tokio::test]
    async fn resolves_human() {
        const DOC: &str = r#"{
            character {
                ... on Human {
                    humanId: id
                    homePlanet
                }
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn ignores_ewok() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                possibleTypes {
                    name
                }
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"possibleTypes": [{"name": "Human"}]}}),
                vec![],
            )),
        );
    }
}

mod external_resolver_enum {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = Database)]
    #[graphql(on Droid = Character::as_droid)]
    enum Character {
        A(Human),
        #[graphql(ignore)]
        B,
    }

    impl Character {
        fn as_droid<'db>(&self, db: &'db Database) -> Option<&'db Droid> {
            if let Self::B = self {
                db.droid.as_ref()
            } else {
                None
            }
        }
    }

    struct Database {
        droid: Option<Droid>,
    }
    impl juniper::Context for Database {}

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(context = Database)]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B,
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);
        let db = Database { droid: None };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);
        let db = Database {
            droid: Some(Droid {
                id: "droid-99".into(),
                primary_function: "run".into(),
            }),
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod external_resolver_enum_variant {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = Database)]
    enum Character {
        A(Human),
        #[graphql(with = Character::as_droid)]
        B(Droid),
    }

    impl Character {
        fn as_droid<'db>(&self, db: &'db Database) -> Option<&'db Droid> {
            if let Self::B(_) = self {
                db.droid.as_ref()
            } else {
                None
            }
        }
    }

    struct Database {
        droid: Option<Droid>,
    }
    impl juniper::Context for Database {}

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(context = Database)]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                Self::Droid => Character::B(Droid {
                    id: "?????".into(),
                    primary_function: "???".into(),
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);
        let db = Database { droid: None };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);
        let db = Database {
            droid: Some(Droid {
                id: "droid-99".into(),
                primary_function: "run".into(),
            }),
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }
}

mod full_featured_enum {
    use super::*;

    /// Rust doc.
    #[derive(GraphQLUnion)]
    #[graphql(name = "MyChar")]
    #[graphql(description = "My character.")]
    #[graphql(context = CustomContext, scalar = DefaultScalarValue)]
    #[graphql(on EwokCustomContext = resolve_ewok)]
    enum Character<T> {
        A(HumanCustomContext),
        #[graphql(with = Character::as_droid)]
        B(DroidCustomContext),
        #[graphql(ignore)]
        C(EwokCustomContext),
        #[graphql(ignore)]
        _State(T),
    }

    impl<T> Character<T> {
        fn as_droid(&self, ctx: &CustomContext) -> Option<&DroidCustomContext> {
            if let CustomContext::Droid = ctx {
                if let Self::B(droid) = self {
                    return Some(droid);
                }
            }
            None
        }
    }

    fn resolve_ewok<'a, T>(
        ewok: &'a Character<T>,
        _: &CustomContext,
    ) -> Option<&'a EwokCustomContext> {
        if let Character::C(ewok) = ewok {
            Some(ewok)
        } else {
            None
        }
    }

    struct QueryRoot;

    #[graphql_object(context = CustomContext, scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn character(&self, ctx: &CustomContext) -> Character<()> {
            match ctx {
                CustomContext::Human => Character::A(HumanCustomContext {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                }),
                CustomContext::Droid => Character::B(DroidCustomContext {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                }),
                CustomContext::Ewok => Character::C(EwokCustomContext {
                    id: "ewok-1".into(),
                    funny: true,
                }),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on HumanCustomContext {
                humanId: id
                homePlanet
            }
            ... on DroidCustomContext {
                droidId: id
                primaryFunction
            }
            ... on EwokCustomContext {
                ewokId: id
                funny
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Human).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Droid).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_ewok() {
        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Ewok).await,
            Ok((
                graphql_value!({"character": {"ewokId": "ewok-1", "funny": true}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_custom_name() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                name
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Ewok).await,
            Ok((graphql_value!({"__type": {"name": "MyChar"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn uses_custom_description() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &CustomContext::Ewok).await,
            Ok((
                graphql_value!({"__type": {"description": "My character."}}),
                vec![],
            )),
        );
    }
}

mod trivial_struct {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = Database)]
    #[graphql(
        on Human = Character::as_human,
        on Droid = Character::as_droid,
    )]
    struct Character {
        id: String,
    }

    impl Character {
        fn as_human<'db>(&self, db: &'db Database) -> Option<&'db Human> {
            if let Some(human) = &db.human {
                if human.id == self.id {
                    return Some(human);
                }
            }
            None
        }

        fn as_droid<'db>(&self, db: &'db Database) -> Option<&'db Droid> {
            if let Some(droid) = &db.droid {
                if droid.id == self.id {
                    return Some(droid);
                }
            }
            None
        }
    }

    struct Database {
        human: Option<Human>,
        droid: Option<Droid>,
    }
    impl juniper::Context for Database {}

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(context = Database)]
    impl QueryRoot {
        fn character(&self) -> Character {
            Character {
                id: match self {
                    Self::Human => "human-32",
                    Self::Droid => "droid-99",
                }
                .into(),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);
        let db = Database {
            human: None,
            droid: Some(Droid {
                id: "droid-99".into(),
                primary_function: "run".into(),
            }),
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn is_graphql_union() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((graphql_value!({"__type": {"kind": "UNION"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn uses_type_name() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                name
            }
        }"#;

        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((graphql_value!({"__type": {"name": "Character"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                description
            }
        }"#;

        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod generic_struct {
    use super::*;

    #[derive(GraphQLUnion)]
    #[graphql(context = Database)]
    #[graphql(on Human = Character::as_human)]
    struct Character<A, B> {
        id: String,
        _s: PhantomData<(A, B)>,
    }

    impl<A, B> Character<A, B> {
        fn as_human<'db>(&self, db: &'db Database) -> Option<&'db Human> {
            if let Some(human) = &db.human {
                if human.id == self.id {
                    return Some(human);
                }
            }
            None
        }
    }

    struct Database {
        human: Option<Human>,
    }
    impl juniper::Context for Database {}

    struct QueryRoot;

    #[graphql_object(context = Database)]
    impl QueryRoot {
        fn character(&self) -> Character<u8, ()> {
            Character {
                id: "human-32".into(),
                _s: PhantomData,
            }
        }
    }

    #[tokio::test]
    async fn resolves_human() {
        const DOC: &str = r#"{
            character {
                ... on Human {
                    humanId: id
                    homePlanet
                }
            }
        }"#;

        let schema = schema(QueryRoot);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_type_name_without_type_params() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                name
            }
        }"#;

        let schema = schema(QueryRoot);
        let db = Database { human: None };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((graphql_value!({"__type": {"name": "Character"}}), vec![])),
        );
    }
}

mod full_featured_struct {
    use super::*;

    /// Rust doc.
    #[derive(GraphQLUnion)]
    #[graphql(name = "MyChar")]
    #[graphql(description = "My character.")]
    #[graphql(context = Database, scalar = DefaultScalarValue)]
    #[graphql(on Human = Character::as_human)]
    #[graphql(on Droid = Character::as_droid)]
    struct Character<T> {
        id: String,
        _s: PhantomData<T>,
    }

    impl<T> Character<T> {
        fn as_human<'db>(&self, db: &'db Database) -> Option<&'db Human> {
            if let Some(human) = &db.human {
                if human.id == self.id {
                    return Some(human);
                }
            }
            None
        }
    }

    impl<T> Character<T> {
        fn as_droid<'db>(&self, db: &'db Database) -> Option<&'db Droid> {
            if let Some(droid) = &db.droid {
                if droid.id == self.id {
                    return Some(droid);
                }
            }
            None
        }
    }

    struct Database {
        human: Option<Human>,
        droid: Option<Droid>,
    }
    impl juniper::Context for Database {}

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object(context = Database, scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn character(&self) -> Character<()> {
            Character {
                id: match self {
                    Self::Human => "human-32",
                    Self::Droid => "droid-99",
                }
                .into(),
                _s: PhantomData,
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: Some(Human {
                id: "human-32".into(),
                home_planet: "earth".into(),
            }),
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);
        let db = Database {
            human: None,
            droid: Some(Droid {
                id: "droid-99".into(),
                primary_function: "run".into(),
            }),
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn uses_custom_name() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                name
            }
        }"#;

        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: None,
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((graphql_value!({"__type": {"name": "MyChar"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn uses_custom_description() {
        const DOC: &str = r#"{
            __type(name: "MyChar") {
                description
            }
        }"#;

        let schema = schema(QueryRoot::Human);
        let db = Database {
            human: None,
            droid: None,
        };

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &db).await,
            Ok((
                graphql_value!({"__type": {"description": "My character."}}),
                vec![],
            )),
        );
    }
}

/// Checks that union with boxed variants resolves okay.
/// See [#845](https://github.com/graphql-rust/juniper/issues/845) for details.
mod issue_845 {
    use std::sync::Arc;

    use super::*;

    #[derive(GraphQLUnion)]
    enum Character {
        A(Box<Human>),
        B(Arc<Droid>),
    }

    enum QueryRoot {
        Human,
        Droid,
    }

    #[graphql_object]
    impl QueryRoot {
        fn character(&self) -> Character {
            match self {
                Self::Human => Character::A(Box::new(Human {
                    id: "human-32".into(),
                    home_planet: "earth".into(),
                })),
                Self::Droid => Character::B(Arc::new(Droid {
                    id: "droid-99".into(),
                    primary_function: "run".into(),
                })),
            }
        }
    }

    const DOC: &str = r#"{
        character {
            ... on Human {
                humanId: id
                homePlanet
            }
            ... on Droid {
                droidId: id
                primaryFunction
            }
        }
    }"#;

    #[tokio::test]
    async fn resolves_human() {
        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"humanId": "human-32", "homePlanet": "earth"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn resolves_droid() {
        let schema = schema(QueryRoot::Droid);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"character": {"droidId": "droid-99", "primaryFunction": "run"}}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn is_graphql_union() {
        const DOC: &str = r#"{
            __type(name: "Character") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot::Human);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "UNION"}}), vec![])),
        );
    }
}
