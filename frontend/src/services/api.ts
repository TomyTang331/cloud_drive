import axios from 'axios';

import type {
    ApiResponse,
    AuthResponse,
    FileListResponse,
    LoginData,
    RegisterData,
    StorageInfo,
    UserProfile
} from '../types';

const API_BASE_URL = 'http://127.0.0.1:3000';

const api = axios.create({
    baseURL: API_BASE_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

api.interceptors.request.use((config) => {
    const token = localStorage.getItem('token');
    if (token) {
        config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
});

// Response interceptor to handle 401 errors
api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Clear invalid token
            localStorage.removeItem('token');
            localStorage.removeItem('user');
            localStorage.removeItem('token_expires_at');

            // Redirect to login page
            if (window.location.pathname !== '/login') {
                window.location.href = '/login';
            }
        }
        return Promise.reject(error);
    }
);

export const authService = {
    register: (data: RegisterData) =>
        api.post<ApiResponse<AuthResponse>>('/api/auth/register', data),

    login: (data: LoginData) =>
        api.post<ApiResponse<AuthResponse>>('/api/auth/login', data),

    getProfile: () =>
        api.get<ApiResponse<UserProfile>>('/api/users/profile'),
};

export const storageService = {
    getStorageInfo: () =>
        api.get<ApiResponse<StorageInfo>>('/api/storage/info'),
};

export const fileService = {
    // List files
    listFiles: (path: string = '/') =>
        api.get<ApiResponse<FileListResponse>>(`/api/files?path=${encodeURIComponent(path)}`),

    // Create folder
    createFolder: (path: string, name: string) =>
        api.post<ApiResponse<unknown>>('/api/files/folder', { path, name }),

    // Delete file/folder
    deleteFile: (fileId: number) =>
        api.delete<ApiResponse<unknown>>(`/api/files?file_id=${fileId}`),

    // Download file
    downloadFile: (fileId: number) =>
        api.get(`/api/files/download?file_id=${fileId}`, { responseType: 'blob' }),

    // Upload file
    uploadFile: (file: File, path: string = '/') => {
        const formData = new FormData();
        formData.append('path', path);
        formData.append('file', file);
        return api.post<ApiResponse<unknown>>('/api/files/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
            },
        });
    },

    // Batch download files
    batchDownload: (fileIds: number[]) =>
        api.post('/api/files/batch-download', { file_ids: fileIds }, { responseType: 'blob' }),
};



export default api;
