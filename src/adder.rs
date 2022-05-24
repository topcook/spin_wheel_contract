#![no_std]
#![feature(generic_associated_types)]

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

const DEFAULT_NUMBER_OF_RESULTS_TO_SHOW: u32 = 20;
const NUMBER_OF_RESULT_TYPES: u32 = 8;

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct ResultType<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
	pub amount: BigUint<M>,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct Result<M: ManagedTypeApi> {
    pub result_type: u32, // ResultType index: 0 ~ 7
	pub user_address: ManagedAddress<M>,
	pub timestamp: u64,
}

#[elrond_wasm::contract]
pub trait SpinWheelGame {

    #[init]
    fn init(&self,
        lottery_output_esdt_token: TokenIdentifier,
        default_input_amount: BigUint,
        result_types: MultiValueManagedVec<ResultType<Self::Api>>
    ) {
        self.default_input_amount().set(&default_input_amount); // 0.05 egld
        self.lottery_output_edst_token().set(&lottery_output_esdt_token);
        
        for result_type in result_types.iter() {
            self.result_types().push(&ResultType{ 
                token_id: result_type.token_id,
                amount: result_type.amount
            });
        }
    }

    #[payable("EGLD")]
    #[endpoint]
    fn do_lottery(
        &self
    ) {
        let input_amount = self.call_value().egld_value();

        require!(
            input_amount >= self.default_input_amount().get(),
            "The payment must match the fixed default input amount"
        );

        let mut rand_source = RandomnessSource::<Self::Api>::new();
        let rand_index = rand_source.next_u32_in_range(0, NUMBER_OF_RESULT_TYPES);

        let caller = self.blockchain().get_caller();

        // esdt token
        let token_id = self.result_types().get((rand_index as u32) as usize).token_id;
        let output_amount = self.result_types().get((rand_index as u32) as usize).amount;

        self.send()
        .direct(&caller, &token_id, 0, &output_amount, b"withdraw esdt token successful");

        // // egld token
        // self.send()
        // .direct_egld(&caller, output_amount, b"withdraw egld successful");

        self.results().push(&Result{
            result_type: rand_index,
	        user_address: caller,
	        timestamp: self.blockchain().get_block_timestamp(),
        });
    }

    #[only_owner]
    #[endpoint(setDefaultInputAmount)]
    fn set_default_input_amount(&self, default_input_amount: BigUint) {
        self.default_input_amount().set(&default_input_amount);
    }

    #[only_owner]
    #[endpoint(setLotteryOutputESDTToken)]
    fn set_lottery_output_edst_token(&self, lottery_output_edst_token: TokenIdentifier) {
        self.lottery_output_edst_token().set(&lottery_output_edst_token);
    }

    #[view(getDefaultInputAmount)]
    #[storage_mapper("defaultInputAmount")]
    fn default_input_amount(&self) -> SingleValueMapper<BigUint>;

    #[view(getLotteryOutputESDTToken)]
    #[storage_mapper("lotteryOutputESDTToken")]
    fn lottery_output_edst_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getResultTypes)]
    #[storage_mapper("resultTypes")]
    fn result_types(&self) -> VecMapper<ResultType<Self::Api>>;

    #[view(getResults)]
    #[storage_mapper("results")]
    fn results(&self) -> VecMapper<Result<Self::Api>>;

    #[view(getRecentLotteryResults)]
    fn get_recent_lottery_results(
        &self,
        opt_number_of_results_to_show: OptionalValue<u32>,
    ) -> MultiValueEncoded<Result<Self::Api>> {
        let mut number_of_results_to_show = match opt_number_of_results_to_show {
            OptionalValue::Some(v) => v,
            OptionalValue::None => DEFAULT_NUMBER_OF_RESULTS_TO_SHOW,
        };

        let number_of_results = self.results().len();
        number_of_results_to_show = core::cmp::min(number_of_results_to_show, number_of_results as u32);

        let mut items_vec = MultiValueEncoded::new();
        for i in 0..number_of_results_to_show {
            items_vec.push(self.results().get((number_of_results as u32 - i) as usize));
        }

        items_vec
    }
}