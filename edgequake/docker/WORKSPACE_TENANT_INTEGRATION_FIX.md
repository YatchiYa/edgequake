# Workspace & Tenant Integration Fix

## Problem Analysis

Based on your error logs, the issue is:

1. **Python Backend** is calling EdgeQuake API but getting 404 errors
2. **EdgeQuake Service** shows no logs (requests not reaching it)
3. **Missing tenant/workspace creation** in EdgeQuake when created in your database

## Root Causes

### 1. EdgeQuake Service Not Running or Wrong URL
Your Python backend logs show:
```
POST /api/v1/tenants/01864b38-029d-4d28-8d44-c9728818d225/workspaces - 404
GET /api/v1/workspaces/d20b0c8d-2363-48a6-abaa-8efd8197e3e4 - 404
```

But EdgeQuake service has no logs, meaning:
- EdgeQuake service might not be running
- Backend is calling wrong URL (e.g., localhost instead of container name)
- Network/routing issue between services

### 2. Backend Service Configuration
Your Python backend needs to:
- Call EdgeQuake API at correct URL (container name in Docker)
- Create tenant in EdgeQuake when creating in database
- Create workspace in EdgeQuake when creating in database
- Upload documents to EdgeQuake after processing

## Solution Steps

### Step 1: Verify EdgeQuake Service is Running

```bash
# Check if EdgeQuake container is running
docker ps | grep edgequake

# Check EdgeQuake logs
docker logs edgequake

# Test EdgeQuake API directly
curl http://localhost:8080/health
curl http://localhost:8080/api/v1/tenants
```

### Step 2: Fix Backend EdgeQuake Service URL

Your Python backend needs to use the correct URL:

**In `backend/services/edgequake_service.py` (or wherever EdgeQuakeService is defined):**

```python
# WRONG (if running in Docker):
EDGEQUAKE_URL = "http://localhost:8080"

# CORRECT (for Docker Compose):
EDGEQUAKE_URL = "http://edgequake:8080"  # Use container name

# OR use environment variable:
EDGEQUAKE_URL = os.getenv("EDGEQUAKE_API_URL", "http://edgequake:8080")
```

### Step 3: Add Backend Service to Docker Compose

Your `docker-compose.yml` needs to include the Python backend:

```yaml
services:
  # EdgeQuake API Server
  edgequake:
    # ... existing config ...

  # Python Backend Service
  backend:
    build:
      context: ../../backend
      dockerfile: Dockerfile
    container_name: backend
    restart: unless-stopped
    ports:
      - "${BACKEND_PORT:-8000}:8000"
    environment:
      - EDGEQUAKE_API_URL=http://edgequake:8080
      - DATABASE_URL=postgresql://user:pass@postgres:5432/dbname
    depends_on:
      - edgequake
      - postgres
    networks:
      - edgequake-network
    volumes:
      - ../../backend:/app

  # ... rest of services ...
```

### Step 4: Fix Tenant Creation Flow

**In your Python backend (e.g., `api_edgequake_routes.py`):**

```python
async def create_tenant_in_db_and_edgequake(tenant_data: dict):
    """Create tenant in both database and EdgeQuake"""
    
    # 1. Create in your database first
    db_tenant = await db.create_tenant(tenant_data)
    
    # 2. Create in EdgeQuake
    try:
        edgequake_response = await edgequake_service.create_tenant({
            "name": tenant_data["name"],
            "slug": tenant_data.get("slug"),
            "description": tenant_data.get("description"),
            "plan": tenant_data.get("plan", "free"),
            "default_llm_model": "gemma3:12b",
            "default_llm_provider": "ollama",
            "default_embedding_model": "text-embedding-3-small",
            "default_embedding_provider": "openai",
            "default_embedding_dimension": 1536
        })
        
        # Store EdgeQuake tenant_id mapping
        await db.update_tenant(db_tenant.id, {
            "edgequake_tenant_id": edgequake_response["id"]
        })
        
    except Exception as e:
        logger.error(f"Failed to create tenant in EdgeQuake: {e}")
        # Decide: rollback DB tenant or continue?
        
    return db_tenant
```

### Step 5: Fix Workspace Creation Flow

```python
async def create_workspace_in_db_and_edgequake(workspace_data: dict, tenant_id: str):
    """Create workspace in both database and EdgeQuake"""
    
    # 1. Get tenant's EdgeQuake ID
    tenant = await db.get_tenant(tenant_id)
    edgequake_tenant_id = tenant.edgequake_tenant_id
    
    if not edgequake_tenant_id:
        # Create tenant in EdgeQuake first
        edgequake_tenant = await edgequake_service.create_tenant({
            "name": tenant.name,
            "slug": tenant.slug
        })
        edgequake_tenant_id = edgequake_tenant["id"]
        await db.update_tenant(tenant_id, {
            "edgequake_tenant_id": edgequake_tenant_id
        })
    
    # 2. Create in your database
    db_workspace = await db.create_workspace(workspace_data)
    
    # 3. Create in EdgeQuake
    try:
        edgequake_response = await edgequake_service.create_workspace(
            edgequake_tenant_id,
            {
                "name": workspace_data["name"],
                "slug": workspace_data.get("slug"),
                "description": workspace_data.get("description"),
                "max_documents": workspace_data.get("max_documents"),
                "llm_model": "gemma3:12b",
                "llm_provider": "ollama",
                "embedding_model": "text-embedding-3-small",
                "embedding_provider": "openai",
                "embedding_dimension": 1536
            }
        )
        
        # Store EdgeQuake workspace_id mapping
        await db.update_workspace(db_workspace.id, {
            "edgequake_workspace_id": edgequake_response["id"]
        })
        
    except Exception as e:
        logger.error(f"Failed to create workspace in EdgeQuake: {e}")
        
    return db_workspace
```

### Step 6: Fix Document Upload Flow

```python
async def upload_document_to_edgequake(
    workspace_id: str,
    file_content: bytes,
    filename: str,
    extracted_text: str
):
    """Upload document to EdgeQuake after processing"""
    
    # 1. Get workspace's EdgeQuake ID
    workspace = await db.get_workspace(workspace_id)
    edgequake_workspace_id = workspace.edgequake_workspace_id
    
    if not edgequake_workspace_id:
        logger.error(f"Workspace {workspace_id} not synced to EdgeQuake")
        return None
    
    # 2. Upload to EdgeQuake
    try:
        response = await edgequake_service.upload_document(
            edgequake_workspace_id,
            {
                "file": (filename, file_content),
                "content": extracted_text,
                "metadata": {
                    "original_filename": filename,
                    "processed_at": datetime.utcnow().isoformat()
                }
            }
        )
        
        return response
        
    except Exception as e:
        logger.error(f"Failed to upload document to EdgeQuake: {e}")
        return None
```

### Step 7: EdgeQuake Service Client Implementation

**In `backend/services/edgequake_service.py`:**

```python
import httpx
import os
from typing import Dict, Any, Optional

class EdgeQuakeService:
    def __init__(self):
        self.base_url = os.getenv("EDGEQUAKE_API_URL", "http://edgequake:8080")
        self.api_prefix = "/api/v1"
        
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
                response = await client.request(
                    method=method,
                    url=url,
                    json=json,
                    files=files
                )
                response.raise_for_status()
                return response.json()
            except httpx.HTTPError as e:
                logger.error(f"[EDGEQUAKE] Error {e.response.status_code}: {e.response.text}")
                raise
    
    async def create_tenant(self, data: Dict) -> Dict:
        """Create tenant in EdgeQuake"""
        logger.info(f"[EDGEQUAKE] Creating tenant: {data['name']}")
        return await self._request("POST", "/tenants", json=data)
    
    async def get_tenant(self, tenant_id: str) -> Optional[Dict]:
        """Get tenant from EdgeQuake"""
        try:
            return await self._request("GET", f"/tenants/{tenant_id}")
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 404:
                return None
            raise
    
    async def create_workspace(self, tenant_id: str, data: Dict) -> Dict:
        """Create workspace in EdgeQuake"""
        logger.info(f"[EDGEQUAKE] Creating workspace in tenant {tenant_id}: {data['name']}")
        return await self._request(
            "POST",
            f"/tenants/{tenant_id}/workspaces",
            json=data
        )
    
    async def get_workspace(self, workspace_id: str) -> Optional[Dict]:
        """Get workspace from EdgeQuake"""
        try:
            return await self._request("GET", f"/workspaces/{workspace_id}")
        except httpx.HTTPStatusError as e:
            if e.response.status_code == 404:
                return None
            raise
    
    async def upload_document(self, workspace_id: str, data: Dict) -> Dict:
        """Upload document to EdgeQuake workspace"""
        logger.info(f"[EDGEQUAKE] Uploading document to workspace {workspace_id}")
        
        # EdgeQuake expects multipart/form-data for file uploads
        files = None
        json_data = None
        
        if "file" in data:
            files = {"file": data["file"]}
            json_data = {k: v for k, v in data.items() if k != "file"}
        else:
            json_data = data
        
        return await self._request(
            "POST",
            f"/workspaces/{workspace_id}/documents",
            json=json_data,
            files=files
        )

# Global instance
edgequake_service = EdgeQuakeService()
```

## Testing the Integration

### 1. Test Tenant Creation

```bash
# Create tenant via your backend
curl -X POST http://localhost:8000/api/v1/tenants \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Tenant",
    "slug": "test-tenant"
  }'

# Verify in EdgeQuake
curl http://localhost:8080/api/v1/tenants

# Check EdgeQuake logs
docker logs edgequake | grep "Created tenant"
```

### 2. Test Workspace Creation

```bash
# Create workspace via your backend
curl -X POST http://localhost:8000/api/v1/tenants/{tenant_id}/workspaces \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Workspace",
    "slug": "test-workspace"
  }'

# Verify in EdgeQuake
curl http://localhost:8080/api/v1/workspaces/{workspace_id}

# Check EdgeQuake logs
docker logs edgequake | grep "Created workspace"
```

### 3. Test Document Upload

```bash
# Upload document via your backend
curl -X POST http://localhost:8000/api/v1/edgequake/workspaces/{workspace_id}/documents/upload \
  -F "file=@test.pdf"

# Check EdgeQuake logs
docker logs edgequake | grep "document"
```

## Debugging Checklist

- [ ] EdgeQuake service is running (`docker ps | grep edgequake`)
- [ ] EdgeQuake health endpoint works (`curl http://localhost:8080/health`)
- [ ] Backend can reach EdgeQuake (`curl http://edgequake:8080/health` from backend container)
- [ ] Backend uses correct URL (`http://edgequake:8080` not `http://localhost:8080`)
- [ ] Tenant creation calls EdgeQuake API
- [ ] Workspace creation calls EdgeQuake API
- [ ] Document upload calls EdgeQuake API
- [ ] EdgeQuake logs show incoming requests
- [ ] Database stores EdgeQuake IDs for mapping

## Common Issues

### Issue 1: 404 Not Found
**Cause:** Backend calling wrong URL or EdgeQuake not running
**Fix:** Use container name `http://edgequake:8080` in Docker network

### Issue 2: Connection Refused
**Cause:** Services not in same Docker network
**Fix:** Add backend to `edgequake-network` in docker-compose.yml

### Issue 3: No Logs in EdgeQuake
**Cause:** Requests not reaching EdgeQuake
**Fix:** Check network connectivity and URL configuration

### Issue 4: Tenant/Workspace Not Found
**Cause:** Not created in EdgeQuake, only in database
**Fix:** Ensure create_tenant/create_workspace calls EdgeQuake API
