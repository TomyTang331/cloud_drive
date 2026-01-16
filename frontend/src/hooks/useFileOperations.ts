import { useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useQueryClient } from '@tanstack/react-query';
import { fileService } from '../services/api';
import { useToast } from './useToast';
import { useProgress } from '../context/ProgressContext';
import { v4 as uuidv4 } from 'uuid';
import { AxiosError } from 'axios';

import { useFilesQuery, useStorageQuery } from './queries/useFiles';
import {
    useCreateFolderMutation,
    useDeleteFileMutation,
    useBatchDeleteMutation,
    useRenameFileMutation,
    useMoveFileMutation,
    useCopyFileMutation
} from './mutations/useFileMutations';

export const useFileOperations = () => {
    const [searchParams, setSearchParams] = useSearchParams();
    const currentPath = searchParams.get('path') || '/';
    const toast = useToast();
    const { addTask, updateTask } = useProgress();
    const queryClient = useQueryClient();

    const { data: files = [], isLoading: loading, refetch: refetchFiles } = useFilesQuery(currentPath);
    const { data: storageInfo = null, refetch: fetchStorageInfo } = useStorageQuery();

    const createFolderMutation = useCreateFolderMutation();
    const deleteFileMutation = useDeleteFileMutation();
    const batchDeleteMutation = useBatchDeleteMutation();
    const renameFileMutation = useRenameFileMutation();
    const moveFileMutation = useMoveFileMutation();
    const copyFileMutation = useCopyFileMutation();

    const setCurrentPath = useCallback((path: string) => {
        setSearchParams({ path });
    }, [setSearchParams]);

    const fetchFiles = useCallback((path?: string) => {
        if (path && path !== currentPath) {
            setCurrentPath(path);
        } else {
            refetchFiles();
        }
    }, [currentPath, setCurrentPath, refetchFiles]);

    const createFolder = async (name: string) => {
        try {
            await createFolderMutation.mutateAsync({ path: currentPath, name });
            return true;
        } catch (error) {
            return false;
        }
    };

    const renameFile = async (fileId: number, newName: string) => {
        try {
            await renameFileMutation.mutateAsync({ fileId, newName });
            return true;
        } catch (error) {
            return false;
        }
    };

    const moveFile = async (fileId: number, destinationPath: string) => {
        try {
            await moveFileMutation.mutateAsync({ fileId, destinationPath });
            return true;
        } catch (error) {
            return false;
        }
    };

    const copyFile = async (fileId: number, destinationPath: string) => {
        try {
            await copyFileMutation.mutateAsync({ fileId, destinationPath });
            return true;
        } catch (error) {
            return false;
        }
    };

    const deleteFile = async (fileId: number) => {
        try {
            await deleteFileMutation.mutateAsync(fileId);
            return true;
        } catch (error) {
            return false;
        }
    };

    const batchDeleteFiles = async (fileIds: number[]) => {
        try {
            await batchDeleteMutation.mutateAsync(fileIds);
            return true;
        } catch (error) {
            return false;
        }
    };

    const uploadFiles = async (filesToUpload: FileList) => {
        if (!filesToUpload || filesToUpload.length === 0) return;

        const filesArray = Array.from(filesToUpload);

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
                    throw error;
                }
            }));

            toast.success('Files uploaded successfully');
            queryClient.invalidateQueries({ queryKey: ['files', currentPath] });
            queryClient.invalidateQueries({ queryKey: ['storage'] });
            return true;
        } catch (error) {
            console.error('Upload batch failed:', error);
            return false;
        }
    };

    const batchDownloadFiles = async (fileIds: number[]) => {
        if (!fileIds || fileIds.length === 0) return;

        const isSingleFile = fileIds.length === 1;
        let fileType = 'file';
        let fileName = 'download';

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

            if (isSingleFile && fileType === 'file') {
                response = await fileService.downloadFile(fileIds[0], (progress) => {
                    updateTask(taskId, { progress });
                });
            } else {
                response = await fileService.batchDownload(fileIds);
                updateTask(taskId, { progress: 50 }); // Indeterminate state or fake progress
            }

            // Create blob link to download
            const blob = response.data;
            const url = window.URL.createObjectURL(blob);

            const link = document.createElement('a');
            link.href = url;

            // Try to get filename from content-disposition
            const contentDisposition = response.headers['content-disposition'];
            let downloadFilename = fileName;

            if (contentDisposition) {
                const filenameStarMatch = contentDisposition.match(/filename\*=UTF-8''([^;\r\n]*)/i);
                if (filenameStarMatch && filenameStarMatch[1]) {
                    downloadFilename = decodeURIComponent(filenameStarMatch[1]);
                } else {
                    const filenameMatch = contentDisposition.match(/filename=['"]?([^;\r\n"']*)['"]?/i);
                    if (filenameMatch && filenameMatch[1]) {
                        downloadFilename = filenameMatch[1];
                    }
                }
            }

            // Ensure zip extension for folders/batch
            if ((!isSingleFile || fileType === 'folder') && !downloadFilename.endsWith('.zip')) {
                downloadFilename += '.zip';
            }

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
                    // Convert blob error to text
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

    const downloadFile = async (fileId: number) => {
        return batchDownloadFiles([fileId]);
    };

    return {
        files,
        loading,
        currentPath,
        setCurrentPath,
        fetchFiles,
        createFolder,
        deleteFile,
        batchDeleteFiles,
        downloadFile,
        uploadFiles,
        batchDownloadFiles,
        renameFile,
        moveFile,
        copyFile,
        storageInfo,
        fetchStorageInfo
    };
};
