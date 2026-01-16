import api from './client';
import type { ApiResponse, FileListResponse, FileItem, CalculateSizeResponse } from '../../types';

export const fileService = {
    listFiles: (path: string = '/') =>
        api.get<ApiResponse<FileListResponse>>(`/api/files?path=${encodeURIComponent(path)}`),

    createFolder: (path: string, name: string) =>
        api.post<ApiResponse<unknown>>('/api/files/folder', { path, name }),

    deleteFile: (fileId: number) =>
        api.delete<ApiResponse<unknown>>(`/api/files?file_id=${fileId}`),

    downloadFile: (fileId: number, onProgress?: (progress: number) => void) =>
        api.get(`/api/files/download?file_id=${fileId}`, {
            responseType: 'blob',
            onDownloadProgress: (progressEvent) => {
                if (onProgress && progressEvent.total) {
                    const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
                    onProgress(progress);
                }
            }
        }),

    uploadFile: (file: File, path: string = '/', onProgress?: (progress: number) => void) => {
        const formData = new FormData();
        formData.append('path', path);
        formData.append('file', file);
        return api.post<ApiResponse<unknown>>('/api/files/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
            },
            onUploadProgress: (progressEvent) => {
                if (onProgress && progressEvent.total) {
                    const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
                    onProgress(progress);
                }
            }
        });
    },

    batchDownload: (fileIds: number[]) =>
        api.post('/api/files/batch-download', { file_ids: fileIds }, { responseType: 'blob' }),

    renameFile: (fileId: number, newName: string) =>
        api.put<ApiResponse<FileItem>>('/api/files/rename', { file_id: fileId, new_name: newName }),

    moveFile: (fileId: number, destinationPath: string) =>
        api.put<ApiResponse<FileItem>>('/api/files/move', { file_id: fileId, destination_path: destinationPath }),

    copyFile: (fileId: number, destinationPath: string) =>
        api.post<ApiResponse<FileItem>>('/api/files/copy', { file_id: fileId, destination_path: destinationPath }),

    calculateSize: (fileIds: number[]) =>
        api.post<ApiResponse<CalculateSizeResponse>>('/api/files/size', { file_ids: fileIds }),
};
