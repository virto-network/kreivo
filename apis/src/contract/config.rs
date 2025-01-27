pub trait Config {
	type AccountId;
	type Balance;
	type Assets;
}

pub trait AssetsConfig {
	type AssetId;
}

pub type AccountIdOf<T> = <T as Config>::AccountId;
pub type BalanceOf<T> = <T as Config>::Balance;
pub type AssetIdOf<T> = <<T as Config>::Assets as AssetsConfig>::AssetId;
