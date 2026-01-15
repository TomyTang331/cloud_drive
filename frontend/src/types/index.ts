export interface ApiResponse<T> {
    code: number;
    message: string;
    request_id: string;
    data: T;
}

export interface RegisterData {
    username: string;
    email: string;
    password: string;
}

export interface LoginData {
    username: string;
    password: string;
}

export interface AuthResponse {
    token: string;
    user_id: number;
    username: string;
    role: string;
}

export interface UserProfile {
    id: number;
    username: string;
    email: string;
    created_at: string;
}

export interface StorageInfo {
    used_bytes: number;
    total_bytes: number;
    usage_percentage: number;
}

export interface FileItem {
    id: number;
    name: string;
    path: string;
    file_type: 'file' | 'folder';
    size_bytes?: number;
    mime_type?: string;
    created_at: string;
    updated_at: string;
    // Permission info
    can_read: boolean;
    can_write: boolean;
    can_delete: boolean;
    is_owner: boolean;
}

export interface FileListResponse {
    files: FileItem[];
    current_path: string;
}

export type SortOption = 'name' | 'size' | 'date';
export type CategoryOption = 'all' | 'recent' | 'images' | 'videos' | 'documents';

export interface RenameRequest {
    file_id: number;
    new_name: string;
}

export interface MoveRequest {
    file_id: number;
    destination_path: string;
}

export interface CopyRequest {
    file_id: number;
    destination_path: string;
}

export interface CalculateSizeRequest {
    file_ids: number[];
}

export interface CalculateSizeResponse {
    total_size_bytes: number;
    file_count: number;
    folder_count: number;
}
