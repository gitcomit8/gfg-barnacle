// integration-example.jsx
// React Integration Example showing the State Fragmentation Bug

import React, { useState } from 'react';
import { useDispatch, useSelector } from 'react-redux';

// ============================================
// Redux Store Setup (Global Store)
// ============================================

// Action Types
const SAVE_PERSONAL_INFO = 'SAVE_PERSONAL_INFO';
const SAVE_SHIPPING_ADDRESS = 'SAVE_SHIPPING_ADDRESS';
const SAVE_BILLING_INFO = 'SAVE_BILLING_INFO';
// NOTICE: No action type for SAVE_SPECIAL_INSTRUCTIONS!
// This is part of the bug - Step 4 is not integrated with Redux
const SAVE_ORDER_REVIEW = 'SAVE_ORDER_REVIEW';
const SET_CURRENT_STEP = 'SET_CURRENT_STEP';

// Initial State
const initialState = {
  currentStep: 1,
  personalInfo: null,
  shippingAddress: null,
  billingInfo: null,
  // specialInstructions: null, // ❌ MISSING! This is the bug!
  orderReview: null,
};

// Reducer
function checkoutReducer(state = initialState, action) {
  switch (action.type) {
    case SAVE_PERSONAL_INFO:
      return { ...state, personalInfo: action.payload };
    case SAVE_SHIPPING_ADDRESS:
      return { ...state, shippingAddress: action.payload };
    case SAVE_BILLING_INFO:
      return { ...state, billingInfo: action.payload };
    // ❌ No case for SAVE_SPECIAL_INSTRUCTIONS!
    case SAVE_ORDER_REVIEW:
      return { ...state, orderReview: action.payload };
    case SET_CURRENT_STEP:
      return { ...state, currentStep: action.payload };
    default:
      return state;
  }
}

// Action Creators
export const savePersonalInfo = (data) => ({
  type: SAVE_PERSONAL_INFO,
  payload: data,
});

export const saveShippingAddress = (data) => ({
  type: SAVE_SHIPPING_ADDRESS,
  payload: data,
});

export const saveBillingInfo = (data) => ({
  type: SAVE_BILLING_INFO,
  payload: data,
});

// ❌ This action creator exists but is NEVER USED in Step4Component!
export const saveSpecialInstructions = (data) => ({
  type: 'SAVE_SPECIAL_INSTRUCTIONS',
  payload: data,
});

export const saveOrderReview = (data) => ({
  type: SAVE_ORDER_REVIEW,
  payload: data,
});

export const setCurrentStep = (step) => ({
  type: SET_CURRENT_STEP,
  payload: step,
});

// ============================================
// Main Checkout Component
// ============================================

export default function CheckoutFlow() {
  const dispatch = useDispatch();
  const currentStep = useSelector((state) => state.checkout.currentStep);

  const goNext = () => {
    if (currentStep < 5) {
      dispatch(setCurrentStep(currentStep + 1));
    }
  };

  const goBack = () => {
    if (currentStep > 1) {
      dispatch(setCurrentStep(currentStep - 1));
      if (currentStep - 1 === 4) {
        console.warn('🐛 BUG TRIGGERED! Going back to Step 4 - local state will be lost!');
      }
    }
  };

  return (
    <div className="checkout-flow">
      <ProgressBar currentStep={currentStep} />
      
      {currentStep === 1 && <Step1PersonalInfo onNext={goNext} />}
      {currentStep === 2 && <Step2ShippingAddress onNext={goNext} onBack={goBack} />}
      {currentStep === 3 && <Step3BillingInfo onNext={goNext} onBack={goBack} />}
      {currentStep === 4 && <Step4SpecialInstructions onNext={goNext} onBack={goBack} />}
      {currentStep === 5 && <Step5OrderReview onBack={goBack} />}
    </div>
  );
}

// ============================================
// Step 1: Personal Info (Uses Redux ✅)
// ============================================

function Step1PersonalInfo({ onNext }) {
  const dispatch = useDispatch();
  const savedData = useSelector((state) => state.checkout.personalInfo);
  
  const [formData, setFormData] = useState(savedData || {
    firstName: '',
    lastName: '',
    email: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    dispatch(savePersonalInfo(formData));
    console.log('✅ Step 1: Saved to Redux Store', formData);
    onNext();
  };

  return (
    <form onSubmit={handleSubmit}>
      <h2>Step 1: Personal Information</h2>
      <p style={{ color: 'green' }}>✅ This step uses Redux (Global Store)</p>
      
      <input
        type="text"
        placeholder="First Name"
        value={formData.firstName}
        onChange={(e) => setFormData({ ...formData, firstName: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="Last Name"
        value={formData.lastName}
        onChange={(e) => setFormData({ ...formData, lastName: e.target.value })}
        required
      />
      
      <input
        type="email"
        placeholder="Email"
        value={formData.email}
        onChange={(e) => setFormData({ ...formData, email: e.target.value })}
        required
      />
      
      <button type="submit">Next →</button>
    </form>
  );
}

// ============================================
// Step 2: Shipping Address (Uses Redux ✅)
// ============================================

function Step2ShippingAddress({ onNext, onBack }) {
  const dispatch = useDispatch();
  const savedData = useSelector((state) => state.checkout.shippingAddress);
  
  const [formData, setFormData] = useState(savedData || {
    street: '',
    city: '',
    postalCode: '',
    country: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    dispatch(saveShippingAddress(formData));
    console.log('✅ Step 2: Saved to Redux Store', formData);
    onNext();
  };

  return (
    <form onSubmit={handleSubmit}>
      <h2>Step 2: Shipping Address</h2>
      <p style={{ color: 'green' }}>✅ This step uses Redux (Global Store)</p>
      
      <input
        type="text"
        placeholder="Street Address"
        value={formData.street}
        onChange={(e) => setFormData({ ...formData, street: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="City"
        value={formData.city}
        onChange={(e) => setFormData({ ...formData, city: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="Postal Code"
        value={formData.postalCode}
        onChange={(e) => setFormData({ ...formData, postalCode: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="Country"
        value={formData.country}
        onChange={(e) => setFormData({ ...formData, country: e.target.value })}
        required
      />
      
      <button type="button" onClick={onBack}>← Back</button>
      <button type="submit">Next →</button>
    </form>
  );
}

// ============================================
// Step 3: Billing Info (Uses Redux ✅)
// ============================================

function Step3BillingInfo({ onNext, onBack }) {
  const dispatch = useDispatch();
  const savedData = useSelector((state) => state.checkout.billingInfo);
  
  const [formData, setFormData] = useState(savedData || {
    cardNumber: '',
    expiry: '',
    cvv: '',
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    dispatch(saveBillingInfo(formData));
    console.log('✅ Step 3: Saved to Redux Store', formData);
    onNext();
  };

  return (
    <form onSubmit={handleSubmit}>
      <h2>Step 3: Billing Information</h2>
      <p style={{ color: 'green' }}>✅ This step uses Redux (Global Store)</p>
      
      <input
        type="text"
        placeholder="Card Number"
        value={formData.cardNumber}
        onChange={(e) => setFormData({ ...formData, cardNumber: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="MM/YY"
        value={formData.expiry}
        onChange={(e) => setFormData({ ...formData, expiry: e.target.value })}
        required
      />
      
      <input
        type="text"
        placeholder="CVV"
        value={formData.cvv}
        onChange={(e) => setFormData({ ...formData, cvv: e.target.value })}
        required
      />
      
      <button type="button" onClick={onBack}>← Back</button>
      <button type="submit">Next →</button>
    </form>
  );
}

// ============================================
// Step 4: Special Instructions (Uses LOCAL STATE ❌)
// ============================================
// 🐛 BUG: This component uses useState instead of Redux!
// When the user navigates back to this step, the component
// remounts and useState resets to the initial empty values!

function Step4SpecialInstructions({ onNext, onBack }) {
  // ❌ BUG: Using local useState instead of Redux!
  // Every time this component mounts, these values reset to empty!
  const [giftMessage, setGiftMessage] = useState('');
  const [deliveryNotes, setDeliveryNotes] = useState('');
  const [signatureRequired, setSignatureRequired] = useState(false);

  const handleSubmit = (e) => {
    e.preventDefault();
    
    // ❌ BUG: This data is NOT saved to Redux!
    // It only exists in local component state!
    const formData = { giftMessage, deliveryNotes, signatureRequired };
    
    console.warn('⚠️ BUG: Step 4 saved to LOCAL STATE (not Redux):', formData);
    console.warn('⚠️ If user clicks Back, this data will be LOST!');
    
    // Notice: We're NOT dispatching to Redux here!
    // dispatch(saveSpecialInstructions(formData)); // ❌ This line is commented out!
    
    onNext();
  };

  return (
    <form onSubmit={handleSubmit}>
      <h2>Step 4: Special Instructions</h2>
      <div style={{ background: '#fff3cd', padding: '15px', borderRadius: '5px', marginBottom: '15px' }}>
        <p style={{ color: '#856404', fontWeight: 'bold' }}>
          🐛 BUG LOCATION: This step uses LOCAL useState instead of Redux!
        </p>
        <p style={{ color: '#856404', fontSize: '14px' }}>
          When you click "Back" to this step, the component remounts and all data is lost!
        </p>
      </div>
      
      <textarea
        placeholder="Gift Message (write something meaningful!)"
        value={giftMessage}
        onChange={(e) => setGiftMessage(e.target.value)}
        rows={4}
      />
      
      <textarea
        placeholder="Delivery Instructions"
        value={deliveryNotes}
        onChange={(e) => setDeliveryNotes(e.target.value)}
        rows={4}
      />
      
      <label>
        <input
          type="checkbox"
          checked={signatureRequired}
          onChange={(e) => setSignatureRequired(e.target.checked)}
        />
        Signature required on delivery
      </label>
      
      <button type="button" onClick={onBack}>← Back</button>
      <button type="submit">Next →</button>
    </form>
  );
}

// ============================================
// Step 5: Order Review (Uses Redux ✅)
// ============================================

function Step5OrderReview({ onBack }) {
  const dispatch = useDispatch();
  const checkoutData = useSelector((state) => state.checkout);
  const [termsAccepted, setTermsAccepted] = useState(false);

  const handleComplete = () => {
    if (!termsAccepted) {
      alert('Please accept the terms and conditions');
      return;
    }

    dispatch(saveOrderReview({ termsAccepted }));
    
    console.error('🐛 CRITICAL BUG: Order submitted with INCOMPLETE data!');
    console.error('Redux Store:', checkoutData);
    console.error('Notice: Step 4 (Special Instructions) is MISSING!');
    
    alert(
      '🐛 Order "Complete" - but Step 4 data is MISSING!\n\n' +
      'This bug would cause:\n' +
      '- Lost gift messages\n' +
      '- Missing delivery instructions\n' +
      '- Incomplete orders\n' +
      '- Customer complaints'
    );
  };

  return (
    <div>
      <h2>Step 5: Review Your Order</h2>
      <p style={{ color: 'green' }}>✅ This step uses Redux (Global Store)</p>
      
      <div style={{ background: '#d4edda', padding: '15px', borderRadius: '5px', marginBottom: '15px' }}>
        <h3>✅ Data from Redux Store</h3>
        <p>Personal Info: {checkoutData.personalInfo ? '✓ Saved' : '✗ Missing'}</p>
        <p>Shipping: {checkoutData.shippingAddress ? '✓ Saved' : '✗ Missing'}</p>
        <p>Billing: {checkoutData.billingInfo ? '✓ Saved' : '✗ Missing'}</p>
      </div>
      
      <div style={{ background: '#f8d7da', padding: '15px', borderRadius: '5px', marginBottom: '15px' }}>
        <h3>❌ Step 4 Data (NOT in Redux Store)</h3>
        <p><strong>Special Instructions: ⚠️ NOT IN REDUX STORE!</strong></p>
        <p style={{ fontSize: '14px', marginTop: '10px' }}>
          <strong>Try it:</strong> Click "Back" to edit Step 4, fill in the form, 
          then come back here. Then go back to Step 4 again - your data will be gone! 🐛
        </p>
      </div>
      
      <pre style={{ background: '#f4f4f4', padding: '15px', borderRadius: '5px', overflow: 'auto' }}>
        {JSON.stringify({
          step1: checkoutData.personalInfo,
          step2: checkoutData.shippingAddress,
          step3: checkoutData.billingInfo,
          step4: 'NOT IN STORE - Only in local state (will be lost!)',
          step5: 'Current step',
        }, null, 2)}
      </pre>
      
      <label style={{ display: 'block', marginBottom: '15px' }}>
        <input
          type="checkbox"
          checked={termsAccepted}
          onChange={(e) => setTermsAccepted(e.target.checked)}
        />
        I accept the terms and conditions
      </label>
      
      <button type="button" onClick={onBack}>
        ← Back (Try going to Step 4 to see the bug!)
      </button>
      <button onClick={handleComplete}>Complete Order</button>
    </div>
  );
}

// ============================================
// Progress Bar Component
// ============================================

function ProgressBar({ currentStep }) {
  const steps = [
    { num: 1, label: 'Personal' },
    { num: 2, label: 'Shipping' },
    { num: 3, label: 'Billing' },
    { num: 4, label: 'Special 🐛' },
    { num: 5, label: 'Review' },
  ];

  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '30px' }}>
      {steps.map((step) => (
        <div
          key={step.num}
          style={{
            flex: 1,
            textAlign: 'center',
            padding: '10px',
            borderBottom: currentStep === step.num ? '3px solid #667eea' : '1px solid #ccc',
            color: currentStep === step.num ? '#667eea' : '#666',
            fontWeight: currentStep === step.num ? 'bold' : 'normal',
          }}
        >
          Step {step.num}: {step.label}
        </div>
      ))}
    </div>
  );
}

// ============================================
// HOW TO FIX THIS BUG
// ============================================

/*
SOLUTION 1: Use Redux for ALL steps (Recommended)

function Step4SpecialInstructions({ onNext, onBack }) {
  const dispatch = useDispatch();
  const savedData = useSelector((state) => state.checkout.specialInstructions);
  
  const [formData, setFormData] = useState(savedData || {
    giftMessage: '',
    deliveryNotes: '',
    signatureRequired: false,
  });

  const handleSubmit = (e) => {
    e.preventDefault();
    dispatch(saveSpecialInstructions(formData)); // ✅ Save to Redux!
    onNext();
  };

  return <form onSubmit={handleSubmit}>...</form>;
}


SOLUTION 2: Use sessionStorage to persist local state

function Step4SpecialInstructions({ onNext, onBack }) {
  const [giftMessage, setGiftMessage] = useState(() => {
    return sessionStorage.getItem('giftMessage') || '';
  });

  useEffect(() => {
    sessionStorage.setItem('giftMessage', giftMessage);
  }, [giftMessage]);

  // Now data survives component remounts
}


SOLUTION 3: Keep all step components mounted (CSS hide/show)

function CheckoutFlow() {
  return (
    <>
      <Step1 style={{ display: currentStep === 1 ? 'block' : 'none' }} />
      <Step2 style={{ display: currentStep === 2 ? 'block' : 'none' }} />
      <Step3 style={{ display: currentStep === 3 ? 'block' : 'none' }} />
      <Step4 style={{ display: currentStep === 4 ? 'block' : 'none' }} />
      <Step5 style={{ display: currentStep === 5 ? 'block' : 'none' }} />
    </>
  );
}
*/
