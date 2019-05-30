use parity_codec::{Decode, Encode};
use runtime_primitives::traits::As;
use support::{
	decl_event, decl_module, decl_storage,
	dispatch::Result, ensure, StorageMap,
};
use system::ensure_signed;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Resource<Balance> {
	level: u32,
	time: u64,
	accrual: Balance,
	rate: u32,
	resource_amount: Balance,
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
		GoldenMines get(golden_mine): map T::AccountId => Resource<T::Balance>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn init(origin) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(!<GoldenMines<T>>::exists(who.clone()), "Player already has account");

			// let block_number = <system::Module<T>>::block_number();
			// let minimum_period = <timestamp::Module<T>>::minimum_period();
			// block_number.as_() * minimum_period.as_();
			let now: u64 = <timestamp::Module<T>>::now().as_();

			let res = Resource {
				level: 0,
				time: now,
				accrual: <T::Balance as As<u64>>::sa(0),
				rate: 0,
				resource_amount: <T::Balance as As<u64>>::sa(100),
			};
			<GoldenMines<T>>::insert(who.clone(), res);

			Self::deposit_event(RawEvent::PlayerCreated(who));
			Ok(())
		}

		pub fn buy_golden_mine(origin) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(<GoldenMines<T>>::exists(who.clone()), "You don't have this resource");

			let mut res = Self::golden_mine(who.clone());
			ensure!(res.level == 0, "Resource already bought");

			let gold_mine_price = <T::Balance as As<u64>>::sa(100);
			ensure!(res.resource_amount >= gold_mine_price, "You don't have enough gold");

			let now: u64 = <timestamp::Module<T>>::now().as_();

			res.level = 1;
			res.time = now;
			res.accrual = <T::Balance as As<u64>>::sa(1);
			res.rate = 1;
			res.resource_amount -= gold_mine_price;

			<GoldenMines<T>>::insert(who.clone(), res);

			Ok(())
		}

		pub fn level_up(origin) -> Result {
			let who = ensure_signed(origin)?;
			ensure!(<GoldenMines<T>>::exists(who.clone()), "You don't have this resource");

			let mut res = Self::golden_mine(who.clone());
			ensure!(res.level != 0, "You have to buy this resource first");

			let now: u64 = <timestamp::Module<T>>::now().as_();

			let next_level_price = <T::Balance as As<u64>>::sa((res.level * res.rate * 100u32) as u64) * res.accrual;

			// Todo: Move to private func
			let tmp = now - res.time;
			let resource_amount = <T::Balance as As<u64>>::sa(tmp / (res.rate as u64)) * res.accrual;

			ensure!(resource_amount >= next_level_price, "You don't have enough gold");

			res.level += 1;
			res.time = now;
			res.accrual += <T::Balance as As<u64>>::sa(1);
			res.resource_amount = resource_amount - next_level_price;

			<GoldenMines<T>>::insert(who.clone(), res);

			Ok(())
		}
	}
}
