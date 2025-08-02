#!/usr/bin/env python3
"""
Simple test script to verify shares compatibility endpoints.
This script tests the basic functionality of the NestJS-compatible shares API.
"""

import requests
import json
import uuid
from datetime import datetime, timezone

# Configuration
BASE_URL = "http://localhost:3000"
SHARES_BASE = f"{BASE_URL}/shares"

def test_create_share_offer():
    """Test creating a share offer"""
    offer_data = {
        "name": "Test Share Offer",
        "description": "A test share offer for compatibility testing",
        "pricePerShare": "1000.00",
        "totalSharesAvailable": "100.00",
        "validFrom": datetime.now(timezone.utc).isoformat(),
        "validUntil": None,
        "minPurchaseQuantity": "1.00",
        "maxPurchaseQuantity": "10.00",
        "settings": {"test": True},
        "metadata": {"createdBy": "test-script"},
        "createdBy": str(uuid.uuid4())
    }
    
    print("Testing POST /shares/offer...")
    try:
        response = requests.post(f"{SHARES_BASE}/offer", json=offer_data)
        print(f"Status: {response.status_code}")
        
        if response.status_code == 201:
            data = response.json()
            print("✅ Share offer created successfully")
            print(f"Offer ID: {data.get('id')}")
            return data.get('id')
        else:
            print("❌ Failed to create share offer")
            print(f"Response: {response.text}")
            return None
    except Exception as e:
        print(f"❌ Error: {e}")
        return None

def test_get_all_offers():
    """Test getting all share offers"""
    print("\nTesting GET /shares/offers...")
    try:
        response = requests.get(f"{SHARES_BASE}/offers")
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print("✅ Successfully retrieved share offers")
            print(f"Total offers: {data.get('total')}")
            print(f"Active offers: {data.get('active')}")
            return True
        else:
            print("❌ Failed to get share offers")
            print(f"Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_subscribe_to_shares(offer_id):
    """Test subscribing to shares (purchase)"""
    if not offer_id:
        print("\nSkipping subscribe test - no offer ID available")
        return False
    
    subscribe_data = {
        "shareOfferId": offer_id,
        "ownerId": str(uuid.uuid4()),
        "ownerType": "member",
        "quantity": "5.00",
        "purchasedBy": str(uuid.uuid4())
    }
    
    print("\nTesting POST /shares/subscribe...")
    try:
        response = requests.post(f"{SHARES_BASE}/subscribe", json=subscribe_data)
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print("✅ Successfully subscribed to shares")
            print(f"Transaction count: {len(data.get('transactions', []))}")
            return True
        else:
            print("❌ Failed to subscribe to shares")
            print(f"Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_get_transactions():
    """Test getting all transactions"""
    print("\nTesting GET /shares/transactions...")
    try:
        response = requests.get(f"{SHARES_BASE}/transactions?page=0&size=10")
        print(f"Status: {response.status_code}")
        
        if response.status_code == 200:
            data = response.json()
            print("✅ Successfully retrieved transactions")
            print(f"Transactions: {len(data.get('transactions', []))}")
            print(f"Total: {data.get('pagination', {}).get('total')}")
            return True
        else:
            print("❌ Failed to get transactions")
            print(f"Response: {response.text}")
            return False
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

def test_api_health():
    """Test if the API is running"""
    print("Testing API health...")
    try:
        response = requests.get(f"{BASE_URL}/api/health")
        if response.status_code == 200:
            print("✅ API is running")
            return True
        else:
            print("❌ API health check failed")
            return False
    except Exception as e:
        print(f"❌ API not accessible: {e}")
        return False

def main():
    """Run all tests"""
    print("🧪 Testing Shares Compatibility API")
    print("=" * 50)
    
    # Check if API is running
    if not test_api_health():
        print("\n❌ Cannot proceed - API is not accessible")
        print("Make sure the server is running on http://localhost:3000")
        return
    
    # Test endpoints
    offer_id = test_create_share_offer()
    test_get_all_offers()
    test_subscribe_to_shares(offer_id)
    test_get_transactions()
    
    print("\n" + "=" * 50)
    print("🎯 Test Summary:")
    print("- Share offer creation: Tested")
    print("- Share offers listing: Tested")
    print("- Share subscription: Tested")
    print("- Transaction history: Tested")
    print("\n💡 Note: These are basic connectivity tests.")
    print("   Full integration tests would require database setup.")

if __name__ == "__main__":
    main()