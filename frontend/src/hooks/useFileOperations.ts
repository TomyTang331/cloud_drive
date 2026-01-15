import { useState, useEffect, useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { AxiosError } from 'axios';
import { fileService, storageService } from '../services/api';
import type { FileItem, StorageInfo } from '../types';
import { useToast } from './useToast';

export const useFileOperations = () => {
    const [files, setFiles] = useState<FileItem[]>([]);
    const [searchParams, setSearchParams] = useSearchParams();
    const currentPath = searchParams.get('path') || '/';
    const [loading, setLoading] = useState(false);
    const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
    const toast = useToast();

    const setCurrentPath = useCallback((path: string) => {
        setSearchParams({ path });
    }, [setSearchParams]);

    const fetchFiles = useCallback(async (path: string) => {
        setLoading(true);
        try {
            const response = await fileService.listFiles(path);
            setFiles(response.data.data.files);
            // Only update if path is different to avoid loops/redundant updates
            if (response.data.data.current_path !== path) {
                setSearchParams({ path: response.data.data.current_path });
            }
        } catch (error) {
            console.error('Failed to fetch files:', error);
            if (error instanceof AxiosError && error.response?.status === 401) {
                console.error('Authentication failed');
            } else {
                toast.error('Failed to load files');
            }
        } finally {
            setLoading(false);
        }
    }, [toast, setSearchParams]);

    const fetchStorageInfo = useCallback(async () => {
        try {
            const response = await storageService.getStorageInfo();
            setStorageInfo(response.data.data);
        } catch (error) {
            console.error('Failed to fetch storage info:', error);
        }
    }, []);

    const createFolder = async (name: string) => {
        if (!name.trim()) return;
        try {
            await fileService.createFolder(currentPath, name);
            toast.success('Folder created successfully');
            fetchFiles(currentPath);
            return true;
        } catch (error) {
            console.error('Failed to create folder:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Failed to create folder: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const renameFile = async (_file: FileItem, newName: string) => {
        if (!newName.trim()) return;
        try {
            // Placeholder for rename API

            toast.info('Rename feature coming soon!');
            fetchFiles(currentPath);
            return true;
        } catch (error) {
            console.error('Failed to rename:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Rename failed: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const deleteFile = async (file: FileItem) => {
        try {
            await fileService.deleteFile(file.id);
            toast.success('File deleted successfully');
            fetchFiles(currentPath);
            fetchStorageInfo(); // Update storage after delete
            return true;
        } catch (error) {
            console.error('Failed to delete file:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Delete failed: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const uploadFiles = async (filesToUpload: FileList) => {
        if (!filesToUpload || filesToUpload.length === 0) return;

        // Show uploading toast
        toast.info(`Uploading ${filesToUpload.length} file(s)...`);

        try {
            const filesArray = Array.from(filesToUpload);
            await Promise.all(filesArray.map(file => fileService.uploadFile(file, currentPath)));
            toast.success('Files uploaded successfully');
            fetchFiles(currentPath);
            fetchStorageInfo();
            return true;
        } catch (error) {
            console.error('Upload failed:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Upload failed: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const batchDownloadFiles = async (fileIds: number[]) => {
        if (!fileIds || fileIds.length === 0) return;

        const isSingleFile = fileIds.length === 1;
        let fileType = 'file';

        // Check if it's a single file and get its type
        if (isSingleFile) {
            const file = files.find(f => f.id === fileIds[0]);
            if (file) {
                fileType = file.file_type;
            }
        }

        toast.info(`Preparing download for ${fileIds.length} file(s)...`);

        try {
            let response;

            // If single file (not folder), use direct download endpoint
            if (isSingleFile && fileType === 'file') {
                response = await fileService.downloadFile(fileIds[0]);
            } else {
                // Multiple files or a folder -> use batch download
                response = await fileService.batchDownload(fileIds);
            }

            // Create a blob link to download
            // response.data is already a Blob because responseType is 'blob'
            const blob = response.data;
            const url = window.URL.createObjectURL(blob);

            const link = document.createElement('a');
            link.href = url;

            // Try to get filename from content-disposition header
            // Headers are case-insensitive in axios, but usually lowercase
            const contentDisposition = response.headers['content-disposition'];
            let filename = `download_${new Date().getTime()}`;

            if (contentDisposition) {
                // Try to match filename* first (RFC 5987)
                const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;\r\n]*)/i);
                if (filenameStarMatch && filenameStarMatch[1]) {
                    filename = decodeURIComponent(filenameStarMatch[1]);
                } else {
                    // Fallback to filename parameter
                    const filenameMatch = contentDisposition.match(/filename=['"]?([^;\r\n"']*)['\"]?/i);
                    if (filenameMatch && filenameMatch[1]) {
                        filename = filenameMatch[1];
                    }
                }
            }

            // Fallback: Add extension if missing and we know it's a zip (batch/folder)
            if ((!isSingleFile || fileType === 'folder') && !filename.endsWith('.zip')) {
                filename += '.zip';
            }

            link.setAttribute('download', filename);
            document.body.appendChild(link);
            link.click();
            link.remove();
            window.URL.revokeObjectURL(url);

            // Success - download initiated
            return true;
        } catch (error) {
            console.error('Download failed:', error);
            let message = 'Unknown error';

            // Handle error message extraction for blob responses
            if (error instanceof AxiosError) {
                if (error.response?.data instanceof Blob) {
                    // If error response is a Blob, try to parse it as JSON
                    try {
                        const text = await error.response.data.text();
                        const jsonError = JSON.parse(text);
                        message = jsonError.message || jsonError.error || 'Download failed';
                    } catch {
                        message = `Download failed: ${error.response?.statusText || 'Server error'}`;
                    }
                } else {
                    message = error.response?.data?.message || error.message || 'Download failed';
                }
            }

            toast.error(`Download failed: ${message}`);
            return false;
        }
    };

    // Initial fetch and navigation
    useEffect(() => {
        fetchFiles(currentPath);
        fetchStorageInfo();
    }, [currentPath, fetchFiles, fetchStorageInfo]);

    return {
        files,
        currentPath,
        loading,
        storageInfo,
        fetchFiles: () => fetchFiles(currentPath), // Expose a version that uses currentPath
        createFolder,
        renameFile,
        deleteFile,
        uploadFiles,
        batchDownloadFiles,
        setCurrentPath // Exposed for navigation
    };
};
