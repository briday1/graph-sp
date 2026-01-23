import dagex
import time

print("\n" + "="*60)
print("SIMPLIFIED DAG DEMO - Proper Data Flow Visualization")
print("="*60)

# ============ DATA SOURCES (Level 0) ============
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

# ============ LEVEL 1 PROCESSORS ============
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
    return {"age_group": group}

def calculate_transaction_stats(inputs):
    """Calculate transaction statistics"""
    transactions = inputs["transactions"]
    purchases = [t for t in transactions if t["type"] == "purchase"]
    refunds = [t for t in transactions if t["type"] == "refund"]
    
    total_spent = sum(t["amount"] for t in purchases)
    total_refunded = sum(t["amount"] for t in refunds)
    net_spent = total_spent - total_refunded
    
    return {
        "total_spent": total_spent,
        "net_spent": net_spent,
        "purchase_count": len(purchases)
    }

def calculate_engagement_score(inputs):
    """Calculate user engagement score"""
    page_views = inputs["page_views"]
    clicks = inputs["clicks"]
    searches = inputs["searches"]
    
    engagement = page_views * 0.3 + clicks * 0.5 + searches * 2.0
    
    return {
        "engagement_score": round(engagement, 2)
    }

# ============ LEVEL 2 PROCESSORS ============
def calculate_risk_score(inputs):
    """Calculate fraud risk score"""
    refunded = inputs.get("total_refunded", 0)
    spent = inputs["total_spent"]
    
    if spent == 0:
        risk = 0.5
    else:
        refund_ratio = refunded / spent if refunded else 0
        risk = refund_ratio * 0.7
    
    return {"risk_score": round(risk, 3)}

def generate_recommendations(inputs):
    """Generate personalized recommendations"""
    age_group = inputs["age_group"]
    premium = inputs["premium"]
    
    recommendations = []
    
    if age_group == "young_adult":
        recommendations.append("trending_items")
    elif age_group == "middle_aged":
        recommendations.append("premium_services")
    
    if premium:
        recommendations.append("early_access")
    
    return {"recommendations": recommendations}

# ============ FINAL AGGREGATION ============
def aggregate_profile(inputs):
    """Aggregate all data into final user profile"""
    profile = {
        "user_id": inputs["user_id"],
        "age_group": inputs["age_group"],
        "engagement": inputs["engagement_score"],
        "risk": inputs["risk_score"],
        "spending": inputs["net_spent"],
        "recommendations": inputs["recommendations"]
    }
    return {"final_profile": profile}

def print_results(inputs):
    """Print the final results"""
    profile = inputs["final_profile"]
    print("\n" + "="*60)
    print("USER PROFILE ANALYSIS RESULTS")
    print("="*60)
    print(f"User ID: {profile['user_id']}")
    print(f"Age Group: {profile['age_group']}")
    print(f"Engagement Score: {profile['engagement']}")
    print(f"Risk Score: {profile['risk']}")
    print(f"Net Spending: ${profile['spending']:.2f}")
    print(f"Recommendations: {', '.join(profile['recommendations'])}")
    print("="*60 + "\n")
    return {"printed": True}

# ============ BUILD THE GRAPH WITH PROPER DATA FLOW ============
print("\n🚀 Building DAG with proper data flow...")

graph = dagex.Graph()

# Level 0: Independent data sources
graph.add(function=generate_user_data, label="UserData", inputs=None,
          outputs=[("user_id", "user_id"), ("age", "age"), ("premium", "premium")])

graph.add(function=generate_transaction_data, label="TransactionData", inputs=None,
          outputs=[("transactions", "transactions")])

graph.add(function=generate_behavior_data, label="BehaviorData", inputs=None,
          outputs=[("page_views", "page_views"), ("clicks", "clicks"), ("searches", "searches")])

# Level 1: Process the data sources
graph.add(function=calculate_age_group, label="AgeGroup",
          inputs=[("age", "age")],
          outputs=[("age_group", "age_group")])

graph.add(function=calculate_transaction_stats, label="TransStats",
          inputs=[("transactions", "transactions")],
          outputs=[("total_spent", "total_spent"), ("net_spent", "net_spent"), 
                   ("purchase_count", "purchase_count")])

graph.add(function=calculate_engagement_score, label="Engagement",
          inputs=[("page_views", "page_views"), ("clicks", "clicks"), ("searches", "searches")],
          outputs=[("engagement_score", "engagement_score")])

# Level 2: Calculate derived metrics
graph.add(function=calculate_risk_score, label="RiskScore",
          inputs=[("total_spent", "total_spent")],
          outputs=[("risk_score", "risk_score")])

graph.add(function=generate_recommendations, label="Recommendations",
          inputs=[("age_group", "age_group"), ("premium", "premium")],
          outputs=[("recommendations", "recommendations")])

# Level 3: Aggregate everything
graph.add(function=aggregate_profile, label="Aggregate",
          inputs=[("user_id", "user_id"), ("age_group", "age_group"),
                  ("engagement_score", "engagement_score"), ("risk_score", "risk_score"),
                  ("net_spent", "net_spent"), ("recommendations", "recommendations")],
          outputs=[("final_profile", "final_profile")])

# Final: Print results
graph.add(function=print_results, label="PrintResults",
          inputs=[("final_profile", "final_profile")],
          outputs=[("printed", "printed")])

# Build the DAG
dag = graph.build()

print("\n📊 Mermaid Diagram:")
mermaid_output = dag.to_mermaid()
print(mermaid_output)

print("\n⚙️  Executing DAG...")
context = dag.execute()

print(f"\n✅ DAG execution complete! Total context keys: {len(context)}")

# Note about the visualization
print("\n" + "="*60)
print("NOTE ABOUT THE MERMAID DIAGRAM:")
print("="*60)
print("Due to how the graph builder currently works, nodes are")
print("connected sequentially based on insertion order, not based")
print("on actual data dependencies. This creates a linear chain")
print("in the diagram even though the data flow is more complex.")
print("\nThe actual data dependencies are:")
print("  • UserData → AgeGroup, Recommendations")
print("  • TransactionData → TransStats → RiskScore")
print("  • BehaviorData → Engagement")
print("  • All converge → Aggregate → PrintResults")
print("="*60)
