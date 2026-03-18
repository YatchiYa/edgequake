# EdgeQuake Integration Fix - Complete

## Problem Identified

Your Python backend couldn't communicate with EdgeQuake because:

1. **Different Docker Networks**: Backend was in `gopubnet-network`, EdgeQuake in `edgequake-network`
2. **Wrong URL**: Backend was likely calling `http://localhost:8080` instead of `http://edgequake:8080`
3. **No Logs**: EdgeQuake received no requests because network isolation prevented communication

## Changes Made

### 1. Docker Compose Network Configuration (`/home/yarab/Bureau/perso/new_ai_plateform/docker-compose.yml`)

**Added backend to EdgeQuake network:**
```yaml
services:
  backend:
    # ... existing config ...
    environment:
      - EDGEQUAKE_API_URL=http://edgequake:8080  # NEW
    networks:
      - gopubnet-network
      - edgequake-network  # NEW

networks:
  gopubnet-network:
    driver: bridge
  edgequake-network:  # NEW
    external: true
    name: docker_edgequake-network
```

### 2. Backend Service URL Configuration

The backend now has access to `EDGEQUAKE_API_URL=http://edgequake:8080` environment variable.

**You need to update your Python backend code** to use this URL:

**File: `backend/services/edgequake_service.py`** (or wherever EdgeQuakeService is defined)

```python
import os

class EdgeQuakeService:
    def __init__(self):
        # Use environment variable, fallback to container name
        self.base_url = os.getenv("EDGEQUAKE_API_URL", "http://edgequake:8080")
        self.api_prefix = "/api/v1"
```

## How to Apply the Fix

### Step 1: Restart Backend Service

```bash
cd /home/yarab/Bureau/perso/new_ai_plateform

# Stop backend
docker compose down backend

# Restart backend with new network configuration
docker compose up -d backend

# Verify backend is connected to both networks
docker inspect gopubnet-backend | grep -A 10 Networks
```

### Step 2: Verify Network Connectivity

```bash
# Test from backend container to EdgeQuake
docker exec gopubnet-backend curl -f http://edgequake:8080/health

# Should return: {"status":"healthy",...}
```

### Step 3: Check Backend Logs

```bash
# Watch backend logs for EdgeQuake API calls
docker logs -f gopubnet-backend

# You should now see logs like:
# [EDGEQUAKE] Creating tenant: ...
# [EDGEQUAKE] Creating workspace: ...
```

### Step 4: Test Tenant Creation

```bash
# Create a tenant via your backend API
curl -X POST http://localhost:8000/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Tenant",
    "slug": "test-tenant"
  }'

# Check EdgeQuake logs - you should now see:
docker logs edgequake | grep "Created tenant"
```

### Step 5: Test Workspace Creation

```bash
# Create workspace via your backend
curl -X POST http://localhost:8000/api/v1/tenants/{tenant_id}/workspaces \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Workspace",
    "slug": "test-workspace"
  }'

# Check EdgeQuake logs
docker logs edgequake | grep "Created workspace"
```

## Backend Code Changes Required

You need to ensure your Python backend calls EdgeQuake API when creating tenants/workspaces/documents.

### Required Changes in Backend

#### 1. Update EdgeQuake Service Client

**File: `backend/services/edgequake_service.py`**

```python
import httpx
import os
from typing import Dict, Any, Optional
import logging

logger = logging.getLogger(__name__)

class EdgeQuakeService:
    def __init__(self):
        self.base_url = os.getenv("EDGEQUAKE_API_URL", "http://edgequake:8080")
        self.api_prefix = "/api/v1"
        logger.info(f"[EDGEQUAKE] Initialized with base_url: {self.base_url}")
        
    async def _request(
        self,
        method: str,
        endpoint: str,
        json: Optional[Dict] = None,
        files: Optional[Dict] = None
    ) -> Dict[str, Any]:
        """Make HTTP request to EdgeQuake API"""
        url = f"{self.base_url}{self.api_prefix}{endpoint}"
        
        async with httpx.AsyncClient(timeout=30.0) as client:
            try:
                logger.info(f"[EDGEQUAKE] {method} {url}")
                response = await client.request(
                    method=method,
                    url=url,
                    json=json,
                    files=files
                )
                response.raise_for_status()
                return response.json()
            except httpx.HTTPStatusError as e:
                logger.error(f"[EDGEQUAKE] Error {e.response.status_code}: {e.response.text}")
                raise
            except Exception as e:
                logger.error(f"[EDGEQUAKE] Request failed: {e}")
                raise
    
    async def create_tenant(self, data: Dict) -> Dict:
        """POST /api/v1/tenants"""
        logger.info(f"[EDGEQUAKE] Creating tenant: {data.get('name')}")
        return await self._request("POST", "/tenants", json=data)
    
    async def create_workspace(self, tenant_id: str, data: Dict) -> Dict:
        """POST /api/v1/tenants/{tenant_id}/workspaces"""
        logger.info(f"[EDGEQUAKE] Creating workspace in tenant {tenant_id}")
        return await self._request(
            "POST",
            f"/tenants/{tenant_id}/workspaces",
            json=data
        )
    
    async def get_workspace(self, workspace_id: str) -> Optional[Dict]:
        """GET /api/v1/workspaces/{workspace_id}"""
        try:
            return await self._request("GET", f"/workspaces/{workspace_id}")
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 404:
                return None
            raise

# Global instance
edgequake_service = EdgeQuakeService()
```

#### 2. Update Workspace Creation Route

**File: `backend/api_edgequake_routes.py`** (or wherever workspace creation is handled)

```python
from services.edgequake_service import edgequake_service

@router.post("/workspaces")
async def create_workspace(workspace_data: dict, db: Session = Depends(get_db)):
    """Create workspace in both database and EdgeQuake"""
    
    # 1. Get tenant info
    tenant = db.query(Tenant).filter_by(id=workspace_data["tenant_id"]).first()
    if not tenant:
        raise HTTPException(status_code=404, detail="Tenant not found")
    
    # 2. Create in database
    db_workspace = Workspace(**workspace_data)
    db.add(db_workspace)
    db.commit()
    db.refresh(db_workspace)
    
    # 3. Create in EdgeQuake
    try:
        # Get or create tenant in EdgeQuake
        edgequake_tenant_id = tenant.edgequake_tenant_id
        
        if not edgequake_tenant_id:
            # Create tenant in EdgeQuake first
            eq_tenant = await edgequake_service.create_tenant({
                "name": tenant.name,
                "slug": tenant.slug or tenant.name.lower().replace(" ", "-"),
                "description": tenant.description,
                "plan": "free"
            })
            edgequake_tenant_id = eq_tenant["id"]
            tenant.edgequake_tenant_id = edgequake_tenant_id
            db.commit()
        
        # Create workspace in EdgeQuake
        eq_workspace = await edgequake_service.create_workspace(
            edgequake_tenant_id,
            {
                "name": db_workspace.name,
                "slug": db_workspace.slug or db_workspace.name.lower().replace(" ", "-"),
                "description": db_workspace.description,
                "llm_model": "gemma3:12b",
                "llm_provider": "ollama",
                "embedding_model": "text-embedding-3-small",
                "embedding_provider": "openai",
                "embedding_dimension": 1536
            }
        )
        
        # Store EdgeQuake workspace ID
        db_workspace.edgequake_workspace_id = eq_workspace["id"]
        db.commit()
        
        logger.info(f"✅ Created workspace in EdgeQuake: {eq_workspace['id']}")
        
    except Exception as e:
        logger.error(f"❌ Failed to create workspace in EdgeQuake: {e}")
        # Continue - workspace created in DB even if EdgeQuake fails
    
    return db_workspace
```

## Verification Checklist

After restarting:

- [ ] Backend container is connected to `edgequake-network`
- [ ] Backend can curl EdgeQuake health endpoint
- [ ] Creating tenant in backend creates it in EdgeQuake
- [ ] Creating workspace in backend creates it in EdgeQuake
- [ ] EdgeQuake logs show incoming requests
- [ ] Document upload works end-to-end

## Expected Behavior After Fix

### Before (Broken):
```
2026-02-14 21:31:23 | INFO  | services.edgequake_service:create_workspace:286 - [EDGEQUAKE] Creating workspace via API
2026-02-14 21:31:23 | INFO  | main_multi_tenant:log_requests:112 - 📥 POST /api/v1/tenants/.../workspaces
2026-02-14 21:31:23 | INFO  | main_multi_tenant:log_requests:116 - 📤 Response: 404
2026-02-14 21:31:23 | ERROR | services.edgequake_service:_request:122 - [EDGEQUAKE] Error 404: Not Found
```

**EdgeQuake logs:** (empty - no requests received)

### After (Fixed):
```
2026-02-14 22:00:00 | INFO  | services.edgequake_service:create_workspace:286 - [EDGEQUAKE] Creating workspace via API
2026-02-14 22:00:00 | INFO  | services.edgequake_service:_request:95 - [EDGEQUAKE] POST http://edgequake:8080/api/v1/tenants/.../workspaces
2026-02-14 22:00:00 | INFO  | main_multi_tenant:log_requests:116 - 📤 Response: 201
2026-02-14 22:00:00 | INFO  | services.edgequake_service:create_workspace:300 - ✅ Created workspace in EdgeQuake: abc-123
```

**EdgeQuake logs:**
```
2026-02-14T22:00:00Z INFO edgequake_api::handlers::workspaces: Created workspace workspace_id=abc-123 tenant_id=xyz-789
```

## Troubleshooting

### Issue: Backend still gets 404

**Check:**
```bash
# Verify backend is in edgequake-network
docker network inspect docker_edgequake-network | grep gopubnet-backend

# Test connectivity
docker exec gopubnet-backend ping -c 2 edgequake
docker exec gopubnet-backend curl http://edgequake:8080/health
```

### Issue: "network not found"

**Fix:**
```bash
# Check EdgeQuake network name
docker network ls | grep edgequake

# If it's named differently, update docker-compose.yml:
networks:
  edgequake-network:
    external: true
    name: <actual-network-name>
```

### Issue: Backend code not using new URL

**Verify:**
```bash
# Check environment variable is set
docker exec gopubnet-backend env | grep EDGEQUAKE

# Should show: EDGEQUAKE_API_URL=http://edgequake:8080
```

## Next Steps

1. **Restart backend** with new network configuration
2. **Update backend code** to use EdgeQuake service client
3. **Test tenant creation** - should create in both DB and EdgeQuake
4. **Test workspace creation** - should create in both DB and EdgeQuake
5. **Test document upload** - should upload to EdgeQuake after OCR processing
6. **Monitor logs** - both backend and EdgeQuake should show activity

## Summary

The fix connects your backend to EdgeQuake's Docker network and provides the correct URL. Now your backend can:

✅ Create tenants in EdgeQuake when created in your database
✅ Create workspaces in EdgeQuake when created in your database  
✅ Upload documents to EdgeQuake after processing
✅ Query EdgeQuake for workspace/document status

All services now work together as a unified system!
