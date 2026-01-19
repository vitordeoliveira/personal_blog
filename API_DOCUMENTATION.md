# Sales Agents API Documentation

This document explains how to integrate with the Sales Agents API to get all agents and chat with them.

## Authentication

All API requests require authentication using an API key. Include your API key in the `Authorization` header:

```
Authorization: Bearer YOUR_API_KEY
```

Or simply:

```
Authorization: YOUR_API_KEY
```

## Base URL

```
https://your-domain.com/api
```

## Endpoints

### 1. Get All Agents

Retrieve a list of all sales agents associated with your API key.

**Endpoint:** `GET /api/agents`

**Headers:**
```
Authorization: Bearer YOUR_API_KEY
Content-Type: application/json
```

**Response:**
```json
{
  "success": true,
  "agents": [
    {
      "id": 1,
      "name": "Sarah Johnson",
      "salesBehavior": "consultative",
      "description": "Experienced sales professional specializing in B2B solutions...",
      "status": "online"
    },
    {
      "id": 2,
      "name": "Michael Chen",
      "salesBehavior": "technical",
      "description": "Technical sales expert with deep product knowledge...",
      "status": "offline"
    }
  ]
}
```

**Example Request (cURL):**
```bash
curl -X GET https://your-domain.com/api/agents \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json"
```

**Example Request (JavaScript/Fetch):**
```javascript
const response = await fetch('https://your-domain.com/api/agents', {
  method: 'GET',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
  }
});

const data = await response.json();
console.log(data.agents);
```

**Example Request (Python):**
```python
import requests

headers = {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
}

response = requests.get('https://your-domain.com/api/agents', headers=headers)
data = response.json()
print(data['agents'])
```

---

### 2. Chat with an Agent

Send a message to a specific sales agent and receive their response. The agent will respond using LangChain with their unique personality, sales behavior, and description.

**Endpoint:** `POST /api/agents/{agentId}/chat`

**Headers:**
```
Authorization: Bearer YOUR_API_KEY
Content-Type: application/json
```

**URL Parameters:**
- `agentId` (integer, required): The ID of the sales agent you want to chat with

**Request Body:**
```json
{
  "message": "Hello! I'm interested in your products."
}
```

**Response:**
```json
{
  "success": true,
  "response": "Hello! I'm Sarah Johnson, and I'd be happy to help you find the right solution for your needs. What specific products or services are you interested in?",
  "agent": {
    "id": 1,
    "name": "Sarah Johnson",
    "salesBehavior": "consultative"
  }
}
```

**Example Request (cURL):**
```bash
curl -X POST https://your-domain.com/api/agents/1/chat \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "Hello! I am interested in your products."
  }'
```

**Example Request (JavaScript/Fetch):**
```javascript
const response = await fetch('https://your-domain.com/api/agents/1/chat', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    message: "Hello! I am interested in your products."
  })
});

const data = await response.json();
console.log(data.response); // Agent's response
```

**Example Request (Python):**
```python
import requests

headers = {
    'Authorization': 'Bearer YOUR_API_KEY',
    'Content-Type': 'application/json'
}

payload = {
    'message': 'Hello! I am interested in your products.'
}

response = requests.post(
    'https://your-domain.com/api/agents/1/chat',
    headers=headers,
    json=payload
)

data = response.json()
print(data['response'])  # Agent's response
```

---

## Error Responses

All endpoints may return the following error responses:

### 401 Unauthorized
```json
{
  "error": "Missing Authorization header"
}
```
or
```json
{
  "error": "Invalid API key"
}
```

### 400 Bad Request
```json
{
  "error": "Message is required"
}
```
or
```json
{
  "error": "Invalid agent ID"
}
```

### 403 Forbidden
```json
{
  "error": "Access denied: Agent does not belong to this API key owner"
}
```

### 404 Not Found
```json
{
  "error": "Sales agent not found or access denied"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error"
}
```

---

## Complete Integration Example

Here's a complete example that gets all agents and then chats with the first online agent:

**JavaScript:**
```javascript
const API_KEY = 'YOUR_API_KEY';
const BASE_URL = 'https://your-domain.com/api';

// Get all agents
async function getAllAgents() {
  const response = await fetch(`${BASE_URL}/agents`, {
    method: 'GET',
    headers: {
      'Authorization': `Bearer ${API_KEY}`,
      'Content-Type': 'application/json'
    }
  });
  
  const data = await response.json();
  return data.agents;
}

// Chat with an agent
async function chatWithAgent(agentId, message) {
  const response = await fetch(`${BASE_URL}/agents/${agentId}/chat`, {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${API_KEY}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ message })
  });
  
  const data = await response.json();
  return data.response;
}

// Example usage
async function example() {
  // Get all agents
  const agents = await getAllAgents();
  console.log('Available agents:', agents);
  
  // Find an online agent
  const onlineAgent = agents.find(agent => agent.status === 'online');
  
  if (onlineAgent) {
    // Chat with the agent
    const response = await chatWithAgent(
      onlineAgent.id,
      "Hello! I'm looking for a solution for my business."
    );
    console.log('Agent response:', response);
  }
}

example();
```

**Python:**
```python
import requests

API_KEY = 'YOUR_API_KEY'
BASE_URL = 'https://your-domain.com/api'

def get_all_agents():
    headers = {
        'Authorization': f'Bearer {API_KEY}',
        'Content-Type': 'application/json'
    }
    response = requests.get(f'{BASE_URL}/agents', headers=headers)
    data = response.json()
    return data['agents']

def chat_with_agent(agent_id, message):
    headers = {
        'Authorization': f'Bearer {API_KEY}',
        'Content-Type': 'application/json'
    }
    payload = {'message': message}
    response = requests.post(
        f'{BASE_URL}/agents/{agent_id}/chat',
        headers=headers,
        json=payload
    )
    data = response.json()
    return data['response']

# Example usage
if __name__ == '__main__':
    # Get all agents
    agents = get_all_agents()
    print('Available agents:', agents)
    
    # Find an online agent
    online_agent = next((a for a in agents if a['status'] == 'online'), None)
    
    if online_agent:
        # Chat with the agent
        response = chat_with_agent(
            online_agent['id'],
            "Hello! I'm looking for a solution for my business."
        )
        print('Agent response:', response)
```

---

## Notes

- **Agent Status**: Agents can have three statuses: `online`, `offline`, or `disabled`. Only agents with status `online` should be used for chat interactions in production.

- **Message Format**: The message should be a plain text string. The agent will respond naturally based on their configured personality, sales behavior, and description.

- **Rate Limiting**: Be mindful of API rate limits. If you encounter rate limiting, implement exponential backoff in your integration.

- **API Key Security**: Never expose your API key in client-side code. Always use it server-side or in secure environments.

- **Response Time**: Chat responses may take a few seconds as they use LangChain with OpenAI's language model to generate contextual responses.


