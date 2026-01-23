import dagex
import random
import time

# ============ DATA SOURCES ============
def generate_user_data(inputs):
    """Generate user profile data"""
    return {
        "user_id": "user_001",
        "age": 28,
        "country": "USA",
        "premium": True
    }

def generate_transaction_data(inputs):
    """Generate transaction history"""
    return {
        "transactions": [
            {"id": 1, "amount": 100.50, "type": "purchase"},
            {"id": 2, "amount": 50.25, "type": "purchase"},
            {"id": 3, "amount": 200.00, "type": "refund"},
            {"id": 4, "amount": 75.80, "type": "purchase"}
        ]
    }

def generate_behavior_data(inputs):
    """Generate user behavior metrics"""
    return {
        "page_views": 150,
        "session_duration": 3600,
        "clicks": 45,
        "searches": 12
    }

# ============ DATA PROCESSORS ============
def calculate_age_group(inputs):
    """Categorize user by age"""
    age = inputs["age"]
    if age < 18:
        group = "minor"
    elif age < 35:
        group = "young_adult"
    elif age < 55:
        group = "middle_aged"
    else:
        group = "senior"
    return {"age_group": group, "age_value": age}

def calculate_transaction_stats(inputs):
    """Calculate transaction statistics"""
    transactions = inputs["transactions"]
    purchases = [t for t in transactions if t["type"] == "purchase"]
    refunds = [t for t in transactions if t["type"] == "refund"]
    
    total_spent = sum(t["amount"] for t in purchases)
    total_refunded = sum(t["amount"] for t in refunds)
    net_spent = total_spent - total_refunded
    avg_transaction = total_spent / len(purchases) if purchases else 0
    
    return {
        "total_spent": total_spent,
        "total_refunded": total_refunded,
        "net_spent": net_spent,
        "avg_transaction": avg_transaction,
        "purchase_count": len(purchases)
    }

def calculate_engagement_score(inputs):
    """Calculate user engagement score"""
    page_views = inputs["page_views"]
    clicks = inputs["clicks"]
    searches = inputs["searches"]
    session_duration = inputs["session_duration"]
    
    # Complex engagement formula
    engagement = (page_views * 0.3 + clicks * 0.5 + searches * 2.0) * (session_duration / 1000)
    
    return {
        "engagement_score": round(engagement, 2),
        "is_highly_engaged": engagement > 200
    }

def apply_premium_multiplier(inputs):
    """Apply premium user multiplier"""
    is_premium = inputs["premium"]
    base_score = inputs["base_score"]
    
    multiplier = 1.5 if is_premium else 1.0
    final_score = base_score * multiplier
    
    return {
        "premium_multiplier": multiplier,
        "adjusted_score": final_score
    }

def calculate_spending_tier(inputs):
    """Determine spending tier"""
    net_spent = inputs["net_spent"]
    
    if net_spent < 100:
        tier = "bronze"
    elif net_spent < 300:
        tier = "silver"
    elif net_spent < 500:
        tier = "gold"
    else:
        tier = "platinum"
    
    return {"spending_tier": tier, "spending_amount": net_spent}

def combine_scores(inputs):
    """Combine engagement and spending scores"""
    engagement = inputs["engagement_score"]
    spending = inputs["net_spent"]
    avg_transaction = inputs["avg_transaction"]
    
    # Weighted combination
    combined = (engagement * 0.4) + (spending * 0.5) + (avg_transaction * 0.1)
    
    return {"base_score": combined}

def calculate_risk_score(inputs):
    """Calculate fraud risk score"""
    refunded = inputs["total_refunded"]
    spent = inputs["total_spent"]
    purchase_count = inputs["purchase_count"]
    
    if spent == 0:
        risk = 0.5
    else:
        refund_ratio = refunded / spent
        # High refund ratio or few purchases = higher risk
        risk = (refund_ratio * 0.7) + ((10 - min(purchase_count, 10)) / 100)
    
    return {
        "risk_score": round(risk, 3),
        "is_high_risk": risk > 0.3
    }

def generate_recommendations(inputs):
    """Generate personalized recommendations"""
    tier = inputs["spending_tier"]
    age_group = inputs["age_group"]
    is_engaged = inputs["is_highly_engaged"]
    is_premium = inputs["premium"]
    
    recommendations = []
    
    if tier in ["gold", "platinum"]:
        recommendations.append("exclusive_products")
    
    if age_group == "young_adult":
        recommendations.append("trending_items")
    elif age_group == "middle_aged":
        recommendations.append("premium_services")
    
    if is_engaged:
        recommendations.append("loyalty_program")
    
    if is_premium:
        recommendations.append("early_access")
    
    return {"recommendations": recommendations}

def create_user_segment(inputs):
    """Create final user segment"""
    age_group = inputs["age_group"]
    tier = inputs["spending_tier"]
    risk = inputs["risk_score"]
    
    segment = f"{age_group}_{tier}_risk_{int(risk*100)}"
    
    return {"user_segment": segment}

def aggregate_final_profile(inputs):
    """Aggregate all data into final user profile"""
    profile = {
        "user_id": inputs["user_id"],
        "segment": inputs["user_segment"],
        "scores": {
            "engagement": inputs["engagement_score"],
            "risk": inputs["risk_score"],
            "final": inputs["adjusted_score"]
        },
        "financial": {
            "tier": inputs["spending_tier"],
            "net_spent": inputs["spending_amount"],
            "avg_transaction": inputs["avg_transaction"]
        },
        "demographics": {
            "age_group": inputs["age_group"],
            "country": inputs["country"],
            "premium": inputs["premium"]
        },
        "recommendations": inputs["recommendations"],
        "risk_flag": inputs["is_high_risk"]
    }
    
    return {"final_profile": profile}

def print_results(inputs):
    """Print the final results"""
    profile = inputs["final_profile"]
    print("\n" + "="*60)
    print("SUPER COMPLICATED USER PROFILE ANALYSIS")
    print("="*60)
    print(f"\nUser ID: {profile['user_id']}")
    print(f"Segment: {profile['segment']}")
    print(f"\nScores:")
    print(f"  - Engagement: {profile['scores']['engagement']}")
    print(f"  - Risk: {profile['scores']['risk']}")
    print(f"  - Final: {profile['scores']['final']:.2f}")
    print(f"\nFinancial:")
    print(f"  - Tier: {profile['financial']['tier']}")
    print(f"  - Net Spent: ${profile['financial']['net_spent']:.2f}")
    print(f"  - Avg Transaction: ${profile['financial']['avg_transaction']:.2f}")
    print(f"\nDemographics:")
    print(f"  - Age Group: {profile['demographics']['age_group']}")
    print(f"  - Country: {profile['demographics']['country']}")
    print(f"  - Premium: {profile['demographics']['premium']}")
    print(f"\nRecommendations: {', '.join(profile['recommendations'])}")
    print(f"High Risk: {profile['risk_flag']}")
    print("="*60 + "\n")
    
    return {"printed": True}

# ============ BUILD THE SUPER COMPLICATED GRAPH ============
print("\n🚀 Building super complicated DAG...")

graph = dagex.Graph()

# Data sources
graph.add(function=generate_user_data, label="UserData", inputs=None,
          outputs=[("user_id", "user_id"), ("age", "age"), 
                   ("country", "country"), ("premium", "premium")])

graph.add(function=generate_transaction_data, label="TransactionData", inputs=None,
          outputs=[("transactions", "transactions")])

graph.add(function=generate_behavior_data, label="BehaviorData", inputs=None,
          outputs=[("page_views", "page_views"), ("clicks", "clicks"),
                   ("searches", "searches"), ("session_duration", "session_duration")])

# First level processing
graph.add(function=calculate_age_group, label="AgeGrouping",
          inputs=[("age", "age")],
          outputs=[("age_group", "age_group"), ("age_value", "age_value")])

graph.add(function=calculate_transaction_stats, label="TransactionStats",
          inputs=[("transactions", "transactions")],
          outputs=[("total_spent", "total_spent"), ("total_refunded", "total_refunded"),
                   ("net_spent", "net_spent"), ("avg_transaction", "avg_transaction"),
                   ("purchase_count", "purchase_count")])

graph.add(function=calculate_engagement_score, label="EngagementScore",
          inputs=[("page_views", "page_views"), ("clicks", "clicks"),
                  ("searches", "searches"), ("session_duration", "session_duration")],
          outputs=[("engagement_score", "engagement_score"), 
                   ("is_highly_engaged", "is_highly_engaged")])

# Second level processing
graph.add(function=calculate_spending_tier, label="SpendingTier",
          inputs=[("net_spent", "net_spent")],
          outputs=[("spending_tier", "spending_tier"), ("spending_amount", "spending_amount")])

graph.add(function=calculate_risk_score, label="RiskScore",
          inputs=[("total_refunded", "total_refunded"), ("total_spent", "total_spent"),
                  ("purchase_count", "purchase_count")],
          outputs=[("risk_score", "risk_score"), ("is_high_risk", "is_high_risk")])

graph.add(function=combine_scores, label="CombineScores",
          inputs=[("engagement_score", "engagement_score"), ("net_spent", "net_spent"),
                  ("avg_transaction", "avg_transaction")],
          outputs=[("base_score", "base_score")])

# Third level processing
graph.add(function=apply_premium_multiplier, label="PremiumMultiplier",
          inputs=[("premium", "premium"), ("base_score", "base_score")],
          outputs=[("premium_multiplier", "premium_multiplier"), 
                   ("adjusted_score", "adjusted_score")])

graph.add(function=generate_recommendations, label="Recommendations",
          inputs=[("spending_tier", "spending_tier"), ("age_group", "age_group"),
                  ("is_highly_engaged", "is_highly_engaged"), ("premium", "premium")],
          outputs=[("recommendations", "recommendations")])

graph.add(function=create_user_segment, label="UserSegment",
          inputs=[("age_group", "age_group"), ("spending_tier", "spending_tier"),
                  ("risk_score", "risk_score")],
          outputs=[("user_segment", "user_segment")])

# Final aggregation
graph.add(function=aggregate_final_profile, label="AggregateProfile",
          inputs=[("user_id", "user_id"), ("user_segment", "user_segment"),
                  ("engagement_score", "engagement_score"), ("risk_score", "risk_score"),
                  ("adjusted_score", "adjusted_score"), ("spending_tier", "spending_tier"),
                  ("spending_amount", "spending_amount"), ("avg_transaction", "avg_transaction"),
                  ("age_group", "age_group"), ("country", "country"), ("premium", "premium"),
                  ("recommendations", "recommendations"), ("is_high_risk", "is_high_risk")],
          outputs=[("final_profile", "final_profile")])

# Print results
graph.add(function=print_results, label="PrintResults",
          inputs=[("final_profile", "final_profile")],
          outputs=[("printed", "printed")])

# Build and visualize
dag = graph.build()

print("\n📊 Mermaid Diagram:")
print(dag.to_mermaid())

# Execute the complex pipeline
print("\n⚙️  Executing super complicated DAG...")
context = dag.execute()

print(f"\n✅ DAG execution complete! Total context keys: {len(context)}")