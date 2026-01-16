import { useMutation, useQueryClient } from '@tanstack/react-query';
import { fileService } from '../../services/api';
import { useToast } from '../../context/ToastContext';

export const useCreateFolderMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: ({ path, name }: { path: string; name: string }) =>
            fileService.createFolder(path, name),
        onSuccess: (_, { path }) => {
            success('Folder created successfully');
            queryClient.invalidateQueries({ queryKey: ['files', path] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to create folder');
        },
    });
};

export const useDeleteFileMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: (fileId: number) => fileService.deleteFile(fileId),
        onSuccess: () => {
            success('File deleted successfully');
            queryClient.invalidateQueries({ queryKey: ['files'] });
            queryClient.invalidateQueries({ queryKey: ['storage'] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to delete file');
        },
    });
};

export const useBatchDeleteMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: async (fileIds: number[]) => {
            const promises = fileIds.map(id => fileService.deleteFile(id));
            await Promise.all(promises);
        },
        onSuccess: () => {
            success('Files deleted successfully');
            queryClient.invalidateQueries({ queryKey: ['files'] });
            queryClient.invalidateQueries({ queryKey: ['storage'] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to delete files');
        },
    });
};

export const useRenameFileMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: ({ fileId, newName }: { fileId: number; newName: string }) =>
            fileService.renameFile(fileId, newName),
        onSuccess: () => {
            success('File renamed successfully');
            queryClient.invalidateQueries({ queryKey: ['files'] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to rename file');
        },
    });
};

export const useMoveFileMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: ({ fileId, destinationPath }: { fileId: number; destinationPath: string }) =>
            fileService.moveFile(fileId, destinationPath),
        onSuccess: () => {
            success('File moved successfully');
            queryClient.invalidateQueries({ queryKey: ['files'] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to move file');
        },
    });
};

export const useCopyFileMutation = () => {
    const queryClient = useQueryClient();
    const { success, error } = useToast();

    return useMutation({
        mutationFn: ({ fileId, destinationPath }: { fileId: number; destinationPath: string }) =>
            fileService.copyFile(fileId, destinationPath),
        onSuccess: () => {
            success('File copied successfully');
            queryClient.invalidateQueries({ queryKey: ['files'] });
            queryClient.invalidateQueries({ queryKey: ['storage'] });
        },
        onError: (err: any) => {
            error(err.response?.data?.message || 'Failed to copy file');
        },
    });
};
