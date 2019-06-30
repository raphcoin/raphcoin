use parity_codec::{Decode, Encode};
use runtime_primitives::traits::As;
use support::{
	decl_event, decl_module, decl_storage,
	dispatch::Result, ensure, StorageMap,
};
use system::ensure_signed;
use rstd::prelude::*;

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum Terrain<Balance> {
	CleanTerrain,
	GoldVein,

	GoldMine(ResourceBuilding<Balance>),
}

impl<Balance> Default for Terrain<Balance> {
	fn default() -> Self {
		Terrain::CleanTerrain
	}
}

#[derive(Encode, Decode, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum ResourceType {
	Gold,
}

impl Default for ResourceType {
	fn default() -> Self {
		ResourceType::Gold
	}
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct ResourceBuilding<Balance> {
	level: u32,
	time: u64,
	accrual: Balance,
	rate: u32,
	resource_type: ResourceType,
	resource_amount: Balance,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Resource<Balance> {
	resource_type: ResourceType,
	resource_amount: Balance,
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct City<Balance> {
	grid: Vec<Vec<Terrain<Balance>>>,
	resources: Vec<Resource<Balance>>,
}

pub trait Trait: balances::Trait + timestamp::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		PlayerCreated(AccountId),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as TemplateModule {
		Cities get(city_of): map T::AccountId => City<T::Balance>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn init(origin) -> Result {
			let who = ensure_signed(origin)?;

			ensure!(!<Cities<T>>::exists(who.clone()), "Player already has account");

			let res = Resource {
				resource_type: ResourceType::Gold,
				resource_amount: <T::Balance as As<u64>>::sa(100),
			};
			// Todo: Randomize gold veins
			let mut grid: Vec<Vec<Terrain<T::Balance>>> = Vec::new();
			for _ in 0..3 {
				let mut rows: Vec<Terrain<T::Balance>> = Vec::new();
				for _ in 0..3 {
					rows.push(Terrain::CleanTerrain);
				}
				grid.push(rows);
			}
			let mut resources = Vec::new();
			resources.push(res);
			let city = City { grid, resources };

			<Cities<T>>::insert(who.clone(), city);

			Self::deposit_event(RawEvent::PlayerCreated(who));
			Ok(())
		}

		pub fn build_gold_mine(origin, row: u32, column: u32) -> Result {
			let who = ensure_signed(origin)?;

			// Todo: Ensure row and column are valid

			ensure!(<Cities<T>>::exists(who.clone()), "You don't have this resource");
			let mut city = Self::city_of(who.clone());

			// Todo: move to resource private impl
			let mut res = city.resources.iter_mut()
				.find(|ref x| x.resource_type == ResourceType::Gold)
				.expect("Failed to retrieve resource");

			let price = <T::Balance as As<u64>>::sa(100);
			ensure!(res.resource_amount >= price, "You don't have enough resource");

			// Todo: Check terrain is GoldVein

			let now: u64 = <timestamp::Module<T>>::now().as_();

			let new_building = ResourceBuilding {
				level: 1,
				time: now,
				accrual: <T::Balance as As<u64>>::sa(1),
				rate: 10,
				resource_type: ResourceType::Gold,
				resource_amount: <T::Balance as As<u64>>::sa(0),
			};
			match city.grid[column as usize][row as usize] {
				Terrain::GoldVein => {
					city.grid[column as usize][row as usize] = Terrain::GoldMine(new_building);
				},
				_ => panic!("Invalid terrain"),
			};

			res.resource_amount -= price;

			<Cities<T>>::insert(who.clone(), city);

			Ok(())
		}

		pub fn level_up_gold_mine(origin, row: u32, column: u32) -> Result {
			let who = ensure_signed(origin)?;

			// Todo: Ensure row and column are valid

			ensure!(<Cities<T>>::exists(who.clone()), "You don't have city");
			let mut city = Self::city_of(who.clone());

			// Todo: move to resource private impl
			let mut res = city.resources.iter_mut()
				.find(|ref x| x.resource_type == ResourceType::Gold)
				.expect("Failed to retrieve resource");

			// Todo: Find GoldMine in city grid
			let mut building = match city.grid[column as usize][row as usize] {
				Terrain::GoldMine(ref x) => x.clone(),
				_ => panic!("Invalid terrain"),
			};

			ensure!(building.level != 0, "You have to buy this building first");

			let now: u64 = <timestamp::Module<T>>::now().as_();

			let next_level_price = <T::Balance as As<u64>>::sa((building.level * building.rate * 100u32) as u64) * building.accrual;
			ensure!(res.resource_amount >= next_level_price, "You don't have enough gold");

			building.level += 1;
			building.time = now;
			building.accrual += <T::Balance as As<u64>>::sa(1);

			res.resource_amount -= next_level_price;

			<Cities<T>>::insert(who.clone(), city);

			Ok(())
		}
	}
}
