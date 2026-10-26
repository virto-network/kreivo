//! `KreivoCallMatcher` knows every call of the runtime that carries another call.
//!
//! The test reads the runtime's metadata and collects every call argument whose type is, or
//! contains, `RuntimeCall` (through `Box`, `Vec`, composites, variants, type parameters, and the
//! `Bounded`/`MaybeHashed` wrappers that hold it by hash or encoded), and every argument that looks
//! like an encoded call (`Vec<u8>` named `call`, `proposal`, ...). Each has to be classified in
//! `CLASSIFICATION`: a new pallet, or a new wrapper call in a later SDK or frame-contrib bump, fails
//! this test until somebody decides what the matcher does with it.

use super::*;

use crate::{
	config::call_matcher::{Classification, CLASSIFICATION},
	RuntimeCall,
};
use alloc::collections::{BTreeMap, BTreeSet};
use frame_support::__private::metadata_ir::MetadataIR;
use scale_info::{form::PortableForm, PortableRegistry, Registry, Type, TypeDef, TypeDefPrimitive};

type Types<'a> = BTreeMap<u32, &'a Type<PortableForm>>;

/// Whether the type `id` is `RuntimeCall` or carries it.
fn carries_call(id: u32, call: u32, types: &Types, visiting: &mut BTreeSet<u32>) -> bool {
	if id == call {
		return true;
	}
	// Recursive types: what is being visited does not carry the call through itself.
	if !visiting.insert(id) {
		return false;
	}
	let ty = types[&id];
	// `Bounded<RuntimeCall>` (a hash, a length and the call encoded) and `MaybeHashed` hold a call
	// that `scale_info` does not follow: they are recognised by name.
	let named = ty
		.path
		.segments
		.last()
		.is_some_and(|name| matches!(name.as_str(), "Bounded" | "MaybeHashed"));
	// A type parameter that is the call, or that `scale_info` was told to skip (`VersionedXcm<Call>`
	// keeps its name, `RuntimeCall`, and no type).
	let by_param = ty.type_params.iter().any(|p| match p.ty {
		Some(t) => carries_call(t.id, call, types, visiting),
		None => matches!(p.name.as_str(), "RuntimeCall" | "Call"),
	});
	let by_def = match &ty.type_def {
		TypeDef::Composite(c) => c.fields.iter().any(|f| carries_call(f.ty.id, call, types, visiting)),
		TypeDef::Variant(v) => v
			.variants
			.iter()
			.flat_map(|v| &v.fields)
			.any(|f| carries_call(f.ty.id, call, types, visiting)),
		TypeDef::Sequence(s) => carries_call(s.type_param.id, call, types, visiting),
		TypeDef::Array(a) => carries_call(a.type_param.id, call, types, visiting),
		TypeDef::Tuple(t) => t.fields.iter().any(|f| carries_call(f.id, call, types, visiting)),
		TypeDef::Compact(c) => carries_call(c.type_param.id, call, types, visiting),
		_ => false,
	};
	visiting.remove(&id);
	named || by_param || by_def
}

/// Whether the type is a `Vec<u8>` (or `[u8; N]`).
fn is_bytes(id: u32, types: &Types) -> bool {
	let is_u8 = |id: u32| matches!(types[&id].type_def, TypeDef::Primitive(TypeDefPrimitive::U8));
	match &types[&id].type_def {
		TypeDef::Sequence(s) => is_u8(s.type_param.id),
		TypeDef::Array(a) => is_u8(a.type_param.id),
		_ => false,
	}
}

/// Every `(pallet, call)` with an argument that carries a call, and every `(pallet, call, argument)`
/// that is a byte string with a name that suggests an encoded call.
fn found() -> (BTreeSet<(String, String)>, BTreeSet<(String, String, String)>) {
	let ir: MetadataIR = Runtime::metadata_ir();
	let mut registry = Registry::new();
	let pallets = ir
		.pallets
		.iter()
		.filter_map(|p| {
			p.calls
				.as_ref()
				.map(|c| (p.name.clone(), registry.register_type(&c.ty).id))
		})
		.collect::<Vec<_>>();
	let call = registry.register_type(&scale_info::meta_type::<RuntimeCall>()).id;
	let portable: PortableRegistry = registry.into();
	let types: Types = portable.types.iter().map(|t| (t.id, &t.ty)).collect();

	let (mut carrying, mut encoded) = (BTreeSet::new(), BTreeSet::new());
	for (pallet, id) in pallets {
		let TypeDef::Variant(calls) = &types[&id].type_def else {
			continue;
		};
		for variant in &calls.variants {
			for field in &variant.fields {
				if carries_call(field.ty.id, call, &types, &mut BTreeSet::new()) {
					carrying.insert((pallet.to_string(), variant.name.to_string()));
				}
				let name = field.name.clone().unwrap_or_default();
				let looks_encoded = ["call", "proposal"].iter().any(|n| name.contains(n)) && !name.contains("hash");
				if looks_encoded && is_bytes(field.ty.id, &types) {
					encoded.insert((pallet.to_string(), variant.name.to_string(), name));
				}
			}
		}
	}
	(carrying, encoded)
}

// Every call of the runtime that carries a call is classified, and nothing else is: a new one fails
// here until it is classified in `config/call_matcher.rs`.
#[test]
fn every_call_that_carries_a_call_is_classified() {
	TestExternalities::default().execute_with(|| {
		let (carrying, encoded) = found();
		let classified = CLASSIFICATION
			.iter()
			.map(|(pallet, call, ..)| (pallet.to_string(), call.to_string()))
			.collect::<BTreeSet<_>>();

		println!("calls that carry a call: {carrying:#?}");
		assert_eq!(
			carrying, classified,
			"the calls that carry a RuntimeCall (left) and the classified ones (right) differ"
		);
		// Calls with an encoded-call argument that is not a `RuntimeCall` (none today): any is a
		// finding to classify.
		assert_eq!(encoded, BTreeSet::new(), "calls with an encoded call as bytes");
	})
}

// The wrappers and the opaque calls of the table are the ones the matcher acts on, and the rest are
// not wrappers for it: the table and `nested_calls` agree.
#[test]
fn the_matcher_does_what_the_classification_says() {
	use crate::config::call_matcher::{samples, KreivoCallMatcher};
	use pallet_pass::{CallMatcher, NestedCalls};

	for (pallet, call, class, reason) in CLASSIFICATION {
		assert!(!reason.is_empty(), "{pallet}::{call} has no reason");
		let Some(sample) = samples().remove(&(*pallet, *call)) else {
			// Only the wrappers and the opaque calls need a sample: they are the ones that act.
			assert_eq!(*class, Classification::None, "{pallet}::{call} has no sample call");
			continue;
		};
		let acts = match KreivoCallMatcher::nested_calls(&sample) {
			NestedCalls::One(_) | NestedCalls::Many(_) | NestedCalls::Iter(_) => Classification::Wrapper,
			NestedCalls::Opaque => Classification::Opaque,
			NestedCalls::None => Classification::None,
		};
		assert_eq!(
			acts, *class,
			"{pallet}::{call} is {class:?} in the table, and the matcher does {acts:?}"
		);
	}
}
