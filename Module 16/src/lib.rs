use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::console;

/// Multi-Step Form State Fragmentation Module
/// 
/// This module demonstrates a critical state management bug in multi-step forms.
/// The bug: Steps 1-3 use a Global Store, Step 4 uses local state, and Step 5 
/// tries to pull from the Global Store again, causing data loss when navigating backward.
///
/// ## The Bug Explained:
/// 1. User fills Steps 1-3 → Data stored in Global Store ✅
/// 2. User fills Step 4 → Data stored in LOCAL state (NOT in Global Store) ❌
/// 3. User proceeds to Step 5 → Pulls data from Global Store (Step 4 data is THERE)
/// 4. User clicks "Back" to Step 4 → Component re-renders with fresh LOCAL state
/// 5. **BUG**: Step 4 data is WIPED because it was never in the Global Store!
///
/// ## Why This Is Difficult:
/// - The bug only appears when navigating backward
/// - Data appears to be saved when moving forward
/// - Different state sources are not immediately obvious
/// - Users lose their work unexpectedly

/// Step data structures
#[derive(Serialize, Deserialize, Clone, Debug)]
#[wasm_bindgen]
pub struct PersonalInfo {
    first_name: String,
    last_name: String,
    email: String,
}

#[wasm_bindgen]
impl PersonalInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(first_name: String, last_name: String, email: String) -> PersonalInfo {
        PersonalInfo {
            first_name,
            last_name,
            email,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn first_name(&self) -> String {
        self.first_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn last_name(&self) -> String {
        self.last_name.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn email(&self) -> String {
        self.email.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[wasm_bindgen]
pub struct ShippingAddress {
    street: String,
    city: String,
    postal_code: String,
    country: String,
}

#[wasm_bindgen]
impl ShippingAddress {
    #[wasm_bindgen(constructor)]
    pub fn new(street: String, city: String, postal_code: String, country: String) -> ShippingAddress {
        ShippingAddress {
            street,
            city,
            postal_code,
            country,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn street(&self) -> String {
        self.street.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn city(&self) -> String {
        self.city.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn postal_code(&self) -> String {
        self.postal_code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn country(&self) -> String {
        self.country.clone()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[wasm_bindgen]
pub struct BillingInfo {
    card_number: String,
    expiry: String,
    cvv: String,
}

#[wasm_bindgen]
impl BillingInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(card_number: String, expiry: String, cvv: String) -> BillingInfo {
        BillingInfo {
            card_number,
            expiry,
            cvv,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn card_number(&self) -> String {
        self.card_number.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn expiry(&self) -> String {
        self.expiry.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn cvv(&self) -> String {
        self.cvv.clone()
    }
}

/// **THE BUGGY PART**: Step 4 - Special Instructions
/// This step uses LOCAL state instead of the Global Store!
/// When the user navigates back to this step, the data is LOST!
#[derive(Serialize, Deserialize, Clone, Debug)]
#[wasm_bindgen]
pub struct SpecialInstructions {
    gift_message: String,
    delivery_notes: String,
    signature_required: bool,
}

#[wasm_bindgen]
impl SpecialInstructions {
    #[wasm_bindgen(constructor)]
    pub fn new(gift_message: String, delivery_notes: String, signature_required: bool) -> SpecialInstructions {
        SpecialInstructions {
            gift_message,
            delivery_notes,
            signature_required,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn gift_message(&self) -> String {
        self.gift_message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn delivery_notes(&self) -> String {
        self.delivery_notes.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn signature_required(&self) -> bool {
        self.signature_required
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[wasm_bindgen]
pub struct OrderReview {
    terms_accepted: bool,
    newsletter_signup: bool,
}

#[wasm_bindgen]
impl OrderReview {
    #[wasm_bindgen(constructor)]
    pub fn new(terms_accepted: bool, newsletter_signup: bool) -> OrderReview {
        OrderReview {
            terms_accepted,
            newsletter_signup,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn terms_accepted(&self) -> bool {
        self.terms_accepted
    }

    #[wasm_bindgen(getter)]
    pub fn newsletter_signup(&self) -> bool {
        self.newsletter_signup
    }
}

/// Global Store for checkout process
/// BUG: Step 4 data (SpecialInstructions) is NOT stored here!
/// It uses local component state instead, causing data loss on back navigation
#[wasm_bindgen]
pub struct CheckoutStore {
    current_step: u8,
    personal_info: Option<PersonalInfo>,
    shipping_address: Option<ShippingAddress>,
    billing_info: Option<BillingInfo>,
    // NOTICE: special_instructions is MISSING from the store!
    // This is intentional - it simulates the bug where Step 4 uses local state
    order_review: Option<OrderReview>,
}

#[wasm_bindgen]
impl CheckoutStore {
    /// Create a new checkout store
    #[wasm_bindgen(constructor)]
    pub fn new() -> CheckoutStore {
        console::log_1(&JsValue::from_str(
            "✅ Global Checkout Store initialized (Steps 1-3, 5)"
        ));
        CheckoutStore {
            current_step: 1,
            personal_info: None,
            shipping_address: None,
            billing_info: None,
            // special_instructions is NOT in the global store!
            order_review: None,
        }
    }

    /// Get current step
    #[wasm_bindgen(getter)]
    pub fn current_step(&self) -> u8 {
        self.current_step
    }

    /// Navigate to next step
    #[wasm_bindgen]
    pub fn next_step(&mut self) {
        if self.current_step < 5 {
            self.current_step += 1;
            console::log_1(&JsValue::from_str(&format!(
                "➡️ Navigating to Step {}", self.current_step
            )));
        }
    }

    /// Navigate to previous step
    /// BUG: When going back to Step 4, local state is reset!
    #[wasm_bindgen]
    pub fn previous_step(&mut self) {
        if self.current_step > 1 {
            self.current_step -= 1;
            console::log_1(&JsValue::from_str(&format!(
                "⬅️ Navigating back to Step {}", self.current_step
            )));
            
            if self.current_step == 4 {
                console::log_1(&JsValue::from_str(
                    "⚠️ BUG TRIGGERED! Returning to Step 4 - Local state will be EMPTY!"
                ));
            }
        }
    }

    /// Step 1: Save personal info to Global Store ✅
    #[wasm_bindgen]
    pub fn save_personal_info(&mut self, first_name: String, last_name: String, email: String) {
        self.personal_info = Some(PersonalInfo::new(first_name, last_name, email));
        console::log_1(&JsValue::from_str(
            "✅ Step 1: Personal info saved to GLOBAL STORE"
        ));
    }

    /// Step 2: Save shipping address to Global Store ✅
    #[wasm_bindgen]
    pub fn save_shipping_address(&mut self, street: String, city: String, postal_code: String, country: String) {
        self.shipping_address = Some(ShippingAddress::new(street, city, postal_code, country));
        console::log_1(&JsValue::from_str(
            "✅ Step 2: Shipping address saved to GLOBAL STORE"
        ));
    }

    /// Step 3: Save billing info to Global Store ✅
    #[wasm_bindgen]
    pub fn save_billing_info(&mut self, card_number: String, expiry: String, cvv: String) {
        self.billing_info = Some(BillingInfo::new(card_number, expiry, cvv));
        console::log_1(&JsValue::from_str(
            "✅ Step 3: Billing info saved to GLOBAL STORE"
        ));
    }

    /// Step 4: Special Instructions - NOT IN GLOBAL STORE!
    /// BUG: This method exists but is NEVER called in the typical flow
    /// Step 4 components use local useState instead!
    #[wasm_bindgen]
    pub fn save_special_instructions_to_store(&mut self, _gift_message: String, _delivery_notes: String, _signature_required: bool) {
        // This method is intentionally unused to simulate the bug
        // In a real buggy application, developers might forget this method exists
        console::log_1(&JsValue::from_str(
            "⚠️ WARNING: This method should save to Global Store but is NOT called!"
        ));
    }

    /// Step 5: Save order review to Global Store ✅
    #[wasm_bindgen]
    pub fn save_order_review(&mut self, terms_accepted: bool, newsletter_signup: bool) {
        self.order_review = Some(OrderReview::new(terms_accepted, newsletter_signup));
        console::log_1(&JsValue::from_str(
            "✅ Step 5: Order review saved to GLOBAL STORE"
        ));
    }

    /// Check if step 1 is complete
    #[wasm_bindgen]
    pub fn is_step1_complete(&self) -> bool {
        self.personal_info.is_some()
    }

    /// Check if step 2 is complete
    #[wasm_bindgen]
    pub fn is_step2_complete(&self) -> bool {
        self.shipping_address.is_some()
    }

    /// Check if step 3 is complete
    #[wasm_bindgen]
    pub fn is_step3_complete(&self) -> bool {
        self.billing_info.is_some()
    }

    /// Check if step 4 is complete
    /// BUG: This always returns false because data is in local state!
    #[wasm_bindgen]
    pub fn is_step4_complete(&self) -> bool {
        // Step 4 data is NOT in the global store!
        console::log_1(&JsValue::from_str(
            "⚠️ BUG: Checking Step 4 completion but data is NOT in Global Store!"
        ));
        false // Always false because special_instructions is not stored!
    }

    /// Check if step 5 is complete
    #[wasm_bindgen]
    pub fn is_step5_complete(&self) -> bool {
        self.order_review.is_some()
    }

    /// Get all data for final submission
    /// BUG: Step 4 data is MISSING!
    #[wasm_bindgen]
    pub fn get_checkout_summary(&self) -> String {
        let summary = format!(
            r#"{{
    "step1_personal_info": {},
    "step2_shipping_address": {},
    "step3_billing_info": {},
    "step4_special_instructions": "⚠️ MISSING - NOT IN GLOBAL STORE!",
    "step5_order_review": {}
}}"#,
            if self.personal_info.is_some() { "✅ Present" } else { "❌ Missing" },
            if self.shipping_address.is_some() { "✅ Present" } else { "❌ Missing" },
            if self.billing_info.is_some() { "✅ Present" } else { "❌ Missing" },
            if self.order_review.is_some() { "✅ Present" } else { "❌ Missing" }
        );
        
        console::log_1(&JsValue::from_str(&format!(
            "🐛 BUG VISIBLE: Checkout summary shows Step 4 data is MISSING!\n{}", summary
        )));
        
        summary
    }

    /// Simulate what happens when user goes back to Step 4
    #[wasm_bindgen]
    pub fn demonstrate_bug(&self) -> String {
        format!(
            r#"🐛 STATE FRAGMENTATION BUG DEMONSTRATION

SCENARIO: User completes all 5 steps, then clicks "Back" from Step 5 to Step 4

WHAT HAPPENS:
1. Steps 1-3: Data is in Global Store ✅
   - Personal Info: {}
   - Shipping Address: {}
   - Billing Info: {}

2. Step 4: Data is in LOCAL COMPONENT STATE ⚠️
   - Special Instructions: NOT IN GLOBAL STORE
   - When component re-renders, useState resets to initial empty state
   - User's carefully written gift message: GONE!
   - Delivery instructions: GONE!

3. Step 5: Tries to read from Global Store ❌
   - Order Review: {}
   - But Step 4 data was never there!

RESULT: User loses all Step 4 data when navigating backward!

ROOT CAUSE: Inconsistent state management
- Steps 1-3, 5 use Global Store (Redux/Zustand)
- Step 4 uses local useState
- Going "Back" causes Step 4 component to remount with fresh useState

FIX: Unify the Source of Truth - store ALL steps in Global Store!"#,
            if self.personal_info.is_some() { "Present" } else { "Missing" },
            if self.shipping_address.is_some() { "Present" } else { "Missing" },
            if self.billing_info.is_some() { "Present" } else { "Missing" },
            if self.order_review.is_some() { "Present" } else { "Missing" }
        )
    }
}

/// Simulates local component state for Step 4
/// This is where the bug lives - state is NOT in the global store
#[wasm_bindgen]
pub struct Step4LocalState {
    gift_message: String,
    delivery_notes: String,
    signature_required: bool,
}

#[wasm_bindgen]
impl Step4LocalState {
    /// Create new local state (simulates useState initial state)
    /// BUG: Every time Step 4 component mounts, this creates EMPTY state
    #[wasm_bindgen(constructor)]
    pub fn new() -> Step4LocalState {
        console::log_1(&JsValue::from_str(
            "⚠️ Step 4 Local State initialized - NOT connected to Global Store!"
        ));
        Step4LocalState {
            gift_message: String::new(),
            delivery_notes: String::new(),
            signature_required: false,
        }
    }

    /// Set gift message in local state
    #[wasm_bindgen]
    pub fn set_gift_message(&mut self, message: String) {
        self.gift_message = message;
        console::log_1(&JsValue::from_str(
            "⚠️ Gift message saved to LOCAL STATE (not Global Store!)"
        ));
    }

    /// Set delivery notes in local state
    #[wasm_bindgen]
    pub fn set_delivery_notes(&mut self, notes: String) {
        self.delivery_notes = notes;
        console::log_1(&JsValue::from_str(
            "⚠️ Delivery notes saved to LOCAL STATE (not Global Store!)"
        ));
    }

    /// Set signature required in local state
    #[wasm_bindgen]
    pub fn set_signature_required(&mut self, required: bool) {
        self.signature_required = required;
        console::log_1(&JsValue::from_str(
            "⚠️ Signature requirement saved to LOCAL STATE (not Global Store!)"
        ));
    }

    /// Get gift message
    #[wasm_bindgen(getter)]
    pub fn gift_message(&self) -> String {
        self.gift_message.clone()
    }

    /// Get delivery notes
    #[wasm_bindgen(getter)]
    pub fn delivery_notes(&self) -> String {
        self.delivery_notes.clone()
    }

    /// Get signature required
    #[wasm_bindgen(getter)]
    pub fn signature_required(&self) -> bool {
        self.signature_required
    }

    /// Demonstrate what happens when component remounts
    #[wasm_bindgen]
    pub fn on_component_remount() -> String {
        console::log_1(&JsValue::from_str(
            "🔄 Step 4 Component Remounting - Local state resets to EMPTY!"
        ));
        String::from("⚠️ COMPONENT REMOUNTED - All local state data LOST!")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Non-WASM tests that check the structure
    #[test]
    fn test_module_structure() {
        // This test runs on non-WASM targets
        // It just verifies the module structure compiles
        assert!(true, "Module structure is valid");
    }

    #[test]
    fn test_data_structures() {
        // Test that data structures can be created
        let personal = PersonalInfo {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            email: "john@example.com".to_string(),
        };
        assert_eq!(personal.first_name, "John");

        let shipping = ShippingAddress {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            postal_code: "12345".to_string(),
            country: "USA".to_string(),
        };
        assert_eq!(shipping.city, "Springfield");

        let billing = BillingInfo {
            card_number: "1234-5678-9012-3456".to_string(),
            expiry: "12/25".to_string(),
            cvv: "123".to_string(),
        };
        assert_eq!(billing.cvv, "123");

        let instructions = SpecialInstructions {
            gift_message: "Test".to_string(),
            delivery_notes: "Notes".to_string(),
            signature_required: true,
        };
        assert_eq!(instructions.gift_message, "Test");
        assert!(instructions.signature_required);

        let review = OrderReview {
            terms_accepted: true,
            newsletter_signup: false,
        };
        assert!(review.terms_accepted);
    }

    // WASM-only tests (these won't run in regular cargo test)
    // To run these: wasm-pack test --headless --firefox
    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::*;

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    fn test_store_missing_step4_data() {
        let store = CheckoutStore::new();
        
        // Step 4 should always report as incomplete in the store
        assert_eq!(store.is_step4_complete(), false);
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    fn test_checkout_flow() {
        let mut store = CheckoutStore::new();
        
        // Fill steps 1-3
        store.save_personal_info(
            "John".to_string(),
            "Doe".to_string(),
            "john@example.com".to_string()
        );
        store.save_shipping_address(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "12345".to_string(),
            "USA".to_string()
        );
        store.save_billing_info(
            "1234-5678-9012-3456".to_string(),
            "12/25".to_string(),
            "123".to_string()
        );
        
        // Steps 1-3 should be complete
        assert!(store.is_step1_complete());
        assert!(store.is_step2_complete());
        assert!(store.is_step3_complete());
        
        // Step 4 is never complete because it uses local state
        assert!(!store.is_step4_complete());
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    fn test_local_state_initialization() {
        let state = Step4LocalState::new();
        
        // Local state should start empty
        assert_eq!(state.gift_message, "");
        assert_eq!(state.delivery_notes, "");
        assert_eq!(state.signature_required, false);
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test]
    fn test_step_navigation() {
        let mut store = CheckoutStore::new();
        
        assert_eq!(store.current_step(), 1);
        
        store.next_step();
        assert_eq!(store.current_step(), 2);
        
        store.next_step();
        store.next_step();
        store.next_step();
        assert_eq!(store.current_step(), 5);
        
        // Going back from step 5 to step 4 triggers the bug
        store.previous_step();
        assert_eq!(store.current_step(), 4);
    }
}
