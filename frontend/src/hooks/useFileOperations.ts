import { useState, useEffect, useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { AxiosError } from 'axios';
import { fileService, storageService } from '../services/api';
import type { FileItem, StorageInfo } from '../types';
import { useToast } from './useToast';

import { useProgress } from '../context/ProgressContext';
import { v4 as uuidv4 } from 'uuid';

export const useFileOperations = () => {
    const [files, setFiles] = useState<FileItem[]>([]);
    const [searchParams, setSearchParams] = useSearchParams();
    const currentPath = searchParams.get('path') || '/';
    const [loading, setLoading] = useState(false);
    const [storageInfo, setStorageInfo] = useState<StorageInfo | null>(null);
    const toast = useToast();
    const { addTask, updateTask } = useProgress();

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

    const renameFile = async (file: FileItem, newName: string) => {
        if (!newName.trim()) return false;
        try {
            await fileService.renameFile(file.id, newName);
            toast.success('File renamed successfully');
            fetchFiles(currentPath);
            return true;
        } catch (error) {
            console.error('Failed to rename:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Rename failed: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const moveFile = async (file: FileItem, destinationPath: string) => {
        try {
            await fileService.moveFile(file.id, destinationPath);
            toast.success('File moved successfully');
            fetchFiles(currentPath);
            return true;
        } catch (error) {
            console.error('Failed to move file:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Move failed: ' + (message || 'Unknown error'));
            return false;
        }
    };

    const copyFile = async (file: FileItem, destinationPath: string) => {
        try {
            await fileService.copyFile(file.id, destinationPath);
            toast.success('File copied successfully');
            fetchFiles(currentPath);
            return true;
        } catch (error) {
            console.error('Failed to copy file:', error);
            const message = error instanceof AxiosError ? error.response?.data?.message : 'Unknown error';
            toast.error('Copy failed: ' + (message || 'Unknown error'));
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

        const filesArray = Array.from(filesToUpload);

        // Create progress tasks for all files
        const uploadTasks = filesArray.map(file => {
            const id = uuidv4();
            addTask({
                id,
                fileName: file.name,
                type: 'upload',
            });
            return { id, file };
        });



        try {
            await Promise.all(uploadTasks.map(async ({ id, file }) => {
                try {
                    updateTask(id, { status: 'active' });
                    await fileService.uploadFile(file, currentPath, (progress) => {
                        updateTask(id, { progress });
                    });
                    updateTask(id, { status: 'completed', progress: 100 });
                } catch (error) {
                    console.error(`Upload failed for ${file.name}:`, error);
                    updateTask(id, {
                        status: 'error',
                        error: error instanceof AxiosError ? error.response?.data?.message || 'Upload failed' : 'Upload failed'
                    });
                    throw error; // Re-throw to trigger toast error if needed, or handle individually
                }
            }));

            toast.success('Files uploaded successfully');
            fetchFiles(currentPath);
            fetchStorageInfo();
            return true;
        } catch (error) {
            console.error('Upload batch failed:', error);
            // Individual errors are handled in the task, but we show a generic toast if something fails
            // toast.error('Some uploads failed'); 
            // Actually, let's not show a generic error toast if we have individual progress errors
            return false;
        }
    };

    const batchDownloadFiles = async (fileIds: number[]) => {
        if (!fileIds || fileIds.length === 0) return;

        const isSingleFile = fileIds.length === 1;
        let fileType = 'file';
        let fileName = 'download';

        // Check if it's a single file and get its type/name
        if (isSingleFile) {
            const file = files.find(f => f.id === fileIds[0]);
            if (file) {
                fileType = file.file_type;
                fileName = file.name;
            }
        } else {
            fileName = `batch_download_${fileIds.length}_files.zip`;
        }

        const taskId = uuidv4();
        addTask({
            id: taskId,
            fileName: fileName,
            type: 'download',
        });
        updateTask(taskId, { status: 'active' });



        try {
            let response;

            // If single file (not folder), use direct download endpoint
            if (isSingleFile && fileType === 'file') {
                response = await fileService.downloadFile(fileIds[0], (progress) => {
                    updateTask(taskId, { progress });
                });
            } else {
                // Multiple files or a folder -> use batch download
                // Batch download might not support progress well if backend doesn't send content-length for zip stream
                // But we can try
                response = await fileService.batchDownload(fileIds);
                // For batch download, we might fake progress or just jump to 100
                updateTask(taskId, { progress: 50 });
            }

            // Create a blob link to download
            const blob = response.data;
            const url = window.URL.createObjectURL(blob);

            const link = document.createElement('a');
            link.href = url;

            // Try to get filename from content-disposition header
            const contentDisposition = response.headers['content-disposition'];
            let downloadFilename = fileName;

            if (contentDisposition) {
                const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;\r\n]*)/i);
                if (filenameStarMatch && filenameStarMatch[1]) {
                    downloadFilename = decodeURIComponent(filenameStarMatch[1]);
                } else {
                    const filenameMatch = contentDisposition.match(/filename=['"]?([^;\r\n"']*)['\"]?/i);
                    if (filenameMatch && filenameMatch[1]) {
                        downloadFilename = filenameMatch[1];
                    }
                }
            }

            // Fallback extension
            if ((!isSingleFile || fileType === 'folder') && !downloadFilename.endsWith('.zip')) {
                downloadFilename += '.zip';
            }

            // Update task name with actual filename
            updateTask(taskId, { fileName: downloadFilename });

            link.setAttribute('download', downloadFilename);
            document.body.appendChild(link);
            link.click();
            link.remove();
            window.URL.revokeObjectURL(url);

            updateTask(taskId, { status: 'completed', progress: 100 });
            return true;
        } catch (error) {
            console.error('Download failed:', error);
            let message = 'Unknown error';

            if (error instanceof AxiosError) {
                if (error.response?.data instanceof Blob) {
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

            updateTask(taskId, { status: 'error', error: message });
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
        moveFile,
        copyFile,
        deleteFile,
        uploadFiles,
        batchDownloadFiles,
        setCurrentPath // Exposed for navigation
    };
};
