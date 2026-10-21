// API client for Штурман PLM microservices
const API_BASE = 'http://localhost';
const SERVICES = {
  auth: 3001,
  parts: 3002,
  documents: 3003,
  bom: 3004,
  changes: 3005,
  ai: 3006,
};

// Auth state (простейшая реализация — в production использовать JWT)
let currentUserId = 'eb649f37-843d-4a53-9854-fb1aac65cd4d'; // tester

export function setUserId(userId: string) {
  currentUserId = userId;
}

export function getUserId(): string {
  return currentUserId;
}

// Generic fetch wrapper
async function apiFetch<T>(
  service: keyof typeof SERVICES,
  path: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE}:${SERVICES[service]}${path}`;
  const headers = {
    'Content-Type': 'application/json',
    'x-user-id': currentUserId,
    ...options.headers,
  };

  const response = await fetch(url, { ...options, headers });
  
  if (!response.ok) {
    const error = await response.text();
    throw new Error(`API Error: ${response.status} - ${error}`);
  }

  return response.json();
}

// Auth API
export const auth = {
  async login(username: string, password: string): Promise<{ user_id: string }> {
    return apiFetch('auth', '/api/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    });
  },

  async register(username: string, password: string, email: string): Promise<{ user_id: string }> {
    return apiFetch('auth', '/api/register', {
      method: 'POST',
      body: JSON.stringify({ username, password, email }),
    });
  },
};

// Parts API
export interface Part {
  id: string;
  number: string;
  name: string;
  description: string | null;
  part_type: string;
  status: string;
  version: string;
  revision: number;
  owner_id: string;
  updated_by: string;
  attributes: Record<string, any>;
  created_at: string;
  updated_at: string;
}

export interface PartsResponse {
  parts: Part[];
  total: number;
  page: number;
  page_size: number;
}

export const parts = {
  async list(): Promise<Part[]> {
    const response: PartsResponse = await apiFetch('parts', '/api/parts');
    return response.parts;
  },

  async get(id: string): Promise<Part> {
    return apiFetch('parts', `/api/parts/${id}`);
  },

  async create(data: {
    number: string;
    name: string;
    description?: string;
    part_type: string;
    status?: string;
    version?: string;
  }): Promise<Part> {
    return apiFetch('parts', '/api/parts', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async update(id: string, data: Partial<Part>): Promise<Part> {
    return apiFetch('parts', `/api/parts/${id}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  },

  async delete(id: string): Promise<void> {
    return apiFetch('parts', `/api/parts/${id}`, { method: 'DELETE' });
  },

  async search(query: string): Promise<Part[]> {
    return apiFetch('parts', `/api/parts/search?q=${encodeURIComponent(query)}`);
  },
};

// Documents API
export interface Document {
  id: string;
  name: string;
  format: string;
  version: string;
  status: string;
  part_id: string | null;
  storage_path: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export const documents = {
  async list(): Promise<Document[]> {
    return apiFetch('documents', '/api/documents');
  },

  async get(id: string): Promise<Document> {
    return apiFetch('documents', `/api/documents/${id}`);
  },

  async create(data: FormData): Promise<Document> {
    const url = `${API_BASE}:${SERVICES.documents}/api/documents`;
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'x-user-id': currentUserId },
      body: data,
    });
    if (!response.ok) throw new Error(`Upload failed: ${response.status}`);
    return response.json();
  },

  async download(id: string): Promise<Blob> {
    const url = `${API_BASE}:${SERVICES.documents}/api/documents/${id}/download`;
    const response = await fetch(url, {
      headers: { 'x-user-id': currentUserId },
    });
    if (!response.ok) throw new Error(`Download failed: ${response.status}`);
    return response.blob();
  },
};

// BOM API
export interface BOM {
  id: string;
  name: string;
  version: string;
  status: string;
  root_part_id: string;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export interface BOMItem {
  id: string;
  bom_id: string;
  part_id: string;
  position: string;
  quantity: number;
  notes: string | null;
  created_at: string;
}

export const bom = {
  async list(): Promise<BOM[]> {
    return apiFetch('bom', '/api/boms');
  },

  async get(id: string): Promise<BOM> {
    return apiFetch('bom', `/api/boms/${id}`);
  },

  async create(data: {
    name: string;
    version: string;
    root_part_id: string;
  }): Promise<BOM> {
    return apiFetch('bom', '/api/boms', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async getItems(bomId: string): Promise<BOMItem[]> {
    return apiFetch('bom', `/api/boms/${bomId}/items`);
  },

  async addItem(bomId: string, data: {
    part_id: string;
    position: string;
    quantity: number;
    notes?: string;
  }): Promise<BOMItem> {
    return apiFetch('bom', `/api/boms/${bomId}/items`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async whereUsed(partId: string): Promise<BOM[]> {
    return apiFetch('bom', `/api/parts/${partId}/where-used`);
  },
};

// Changes (ECO) API
export interface Change {
  id: string;
  number: string;
  title: string;
  description: string | null;
  type: string;
  status: string;
  priority: number;
  created_by: string;
  created_at: string;
  updated_at: string;
}

export const changes = {
  async list(): Promise<Change[]> {
    return apiFetch('changes', '/api/changes');
  },

  async get(id: string): Promise<Change> {
    return apiFetch('changes', `/api/changes/${id}`);
  },

  async create(data: {
    title: string;
    description?: string;
    type: string;
    priority?: number;
  }): Promise<Change> {
    return apiFetch('changes', '/api/changes', {
      method: 'POST',
      body: JSON.stringify(data),
    });
  },

  async submit(id: string): Promise<void> {
    return apiFetch('changes', `/api/changes/${id}/submit`, { method: 'POST' });
  },

  async approve(id: string, comment?: string): Promise<void> {
    return apiFetch('changes', `/api/changes/${id}/approve`, {
      method: 'POST',
      body: JSON.stringify({ comment }),
    });
  },

  async reject(id: string, comment: string): Promise<void> {
    return apiFetch('changes', `/api/changes/${id}/reject`, {
      method: 'POST',
      body: JSON.stringify({ comment }),
    });
  },
};

// AI API
export interface AITask {
  id: string;
  task_type: string;
  status: string;
  priority: number;
  input_data: any;
  result_data: any | null;
  error: string | null;
  created_by: string;
  created_at: string;
  started_at: string | null;
  completed_at: string | null;
}

export const ai = {
  async createTask(taskType: string, inputData: any, priority = 5): Promise<AITask> {
    return apiFetch('ai', '/api/tasks', {
      method: 'POST',
      body: JSON.stringify({
        task_type: taskType,
        input_data: inputData,
        priority,
      }),
    });
  },

  async getTask(id: string): Promise<AITask> {
    return apiFetch('ai', `/api/tasks/${id}`);
  },

  async listTasks(): Promise<AITask[]> {
    return apiFetch('ai', '/api/tasks');
  },

  async cancelTask(id: string): Promise<void> {
    return apiFetch('ai', `/api/tasks/${id}/cancel`, { method: 'POST' });
  },

  // Shortcuts for specific AI tasks
  async semanticSearch(query: string, filters?: any): Promise<Part[]> {
    const task = await this.createTask('semantic_search', { query, filters }, 8);
    // Poll for completion
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 500));
      const updated = await this.getTask(task.id);
      if (updated.status === 'completed') {
        return updated.result_data?.results || [];
      }
      if (updated.status === 'failed') {
        throw new Error(updated.error || 'Task failed');
      }
    }
    throw new Error('Task timeout');
  },

  async analyzeCad(documentId: string): Promise<any> {
    const task = await this.createTask('cad_analysis', { document_id: documentId }, 7);
    for (let i = 0; i < 30; i++) {
      await new Promise(r => setTimeout(r, 1000));
      const updated = await this.getTask(task.id);
      if (updated.status === 'completed') return updated.result_data;
      if (updated.status === 'failed') throw new Error(updated.error || 'Task failed');
    }
    throw new Error('Task timeout');
  },

  async validateBom(bomId: string): Promise<any> {
    const task = await this.createTask('bom_validation', { bom_id: bomId }, 8);
    for (let i = 0; i < 20; i++) {
      await new Promise(r => setTimeout(r, 500));
      const updated = await this.getTask(task.id);
      if (updated.status === 'completed') return updated.result_data;
      if (updated.status === 'failed') throw new Error(updated.error || 'Task failed');
    }
    throw new Error('Task timeout');
  },
};
