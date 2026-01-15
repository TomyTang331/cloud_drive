import React, { useEffect, useState } from 'react';
import { fileService } from '../../services/api';
import type { FileItem } from '../../types';
import { useToast } from '../../hooks/useToast';
import './ImagePreviewModal.less';

interface ImagePreviewModalProps {
    isOpen: boolean;
    onClose: () => void;
    file: FileItem | null;
    onNext?: () => void;
    onPrev?: () => void;
    hasNext?: boolean;
    hasPrev?: boolean;
    onDelete?: (file: FileItem) => void;
}

const ImagePreviewModal: React.FC<ImagePreviewModalProps> = ({
    isOpen,
    onClose,
    file,
    onNext,
    onPrev,
    hasNext,
    hasPrev,
    onDelete
}) => {
    const toast = useToast();
    const [imageUrl, setImageUrl] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [scale, setScale] = useState(1);
    const [rotation, setRotation] = useState(0);
    const [showDetails, setShowDetails] = useState(false);

    useEffect(() => {
        if (!isOpen || !file) {
            setImageUrl(null);
            setError(null);
            setScale(1);
            setRotation(0);
            setShowDetails(false);
            return;
        }

        const fetchImage = async () => {
            setLoading(true);
            setError(null);
            setScale(1);
            setRotation(0);
            try {
                const response = await fileService.downloadFile(file.id);
                const url = URL.createObjectURL(response.data);
                setImageUrl(url);
            } catch (err: any) {
                console.error('Failed to load image:', err);
                setError(`Failed to load image: ${err.message || 'Unknown error'}`);
            } finally {
                setLoading(false);
            }
        };

        fetchImage();

        // Cleanup function
        return () => {
            if (imageUrl) {
                URL.revokeObjectURL(imageUrl);
            }
        };
    }, [isOpen, file]); // Re-run if file changes

    // Keyboard navigation
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'ArrowRight' && hasNext && onNext) {
                onNext();
            } else if (e.key === 'ArrowLeft' && hasPrev && onPrev) {
                onPrev();
            } else if (e.key === 'Escape') {
                onClose();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, hasNext, hasPrev, onNext, onPrev, onClose]);

    const handleDownload = async () => {
        if (!file) return;
        try {
            const response = await fileService.downloadFile(file.id);
            const url = URL.createObjectURL(response.data);
            const a = document.createElement('a');
            a.href = url;
            a.download = file.name;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
        } catch (err) {
            console.error('Download failed:', err);
            toast.error('Failed to download file');
        }
    };

    const handleDelete = () => {
        if (file && onDelete) {
            onDelete(file);
            onClose();
        }
    };

    const handleZoomIn = () => setScale(prev => Math.min(prev + 0.25, 3));
    const handleZoomOut = () => setScale(prev => Math.max(prev - 0.25, 0.5));
    const handleRotate = () => setRotation(prev => (prev + 90) % 360);
    const toggleDetails = () => setShowDetails(prev => !prev);

    if (!isOpen || !file) return null;

    return (
        <div className="image-preview-overlay" onClick={onClose}>
            <div className={`image-preview-container ${showDetails ? 'with-details' : ''}`} onClick={(e) => e.stopPropagation()}>
                <div className="preview-main">
                    <div className="preview-header">
                        <div className="preview-title">
                            <h3>{file.name}</h3>
                            <span className="preview-meta">
                                {new Date(file.created_at).toLocaleString()}
                            </span>
                        </div>
                        <div className="header-actions">
                            <button className={`action-icon-btn ${showDetails ? 'active' : ''}`} onClick={toggleDetails} title="Info">
                                ℹ️
                            </button>
                            <button className="preview-close-btn" onClick={onClose} title="Close">
                                ✕
                            </button>
                        </div>
                    </div>

                    <div className="preview-body">
                        {hasPrev && (
                            <button className="nav-btn prev" onClick={onPrev} title="Previous">
                                ‹
                            </button>
                        )}

                        <div className="image-wrapper">
                            {loading ? (
                                <div className="preview-loading">
                                    <div className="spinner"></div>
                                    <p>Loading image...</p>
                                </div>
                            ) : error ? (
                                <div className="preview-error">
                                    <div className="error-icon">⚠️</div>
                                    <p>{error}</p>
                                </div>
                            ) : imageUrl ? (
                                <img
                                    src={imageUrl}
                                    alt={file.name}
                                    className="preview-image"
                                    style={{
                                        transform: `scale(${scale}) rotate(${rotation}deg)`,
                                        transition: 'transform 0.2s ease'
                                    }}
                                />
                            ) : null}
                        </div>

                        {hasNext && (
                            <button className="nav-btn next" onClick={onNext} title="Next">
                                ›
                            </button>
                        )}
                    </div>

                    <div className="preview-footer">
                        <div className="tool-group">
                            <button className="footer-btn icon-only" onClick={handleZoomOut} title="Zoom Out" disabled={scale <= 0.5}>
                                ➖
                            </button>
                            <span className="zoom-level">{Math.round(scale * 100)}%</span>
                            <button className="footer-btn icon-only" onClick={handleZoomIn} title="Zoom In" disabled={scale >= 3}>
                                ➕
                            </button>
                            <div className="divider"></div>
                            <button className="footer-btn icon-only" onClick={handleRotate} title="Rotate">
                                🔄
                            </button>
                        </div>

                        <div className="action-group">
                            <button className="footer-btn" onClick={handleDownload} title="Download">
                                <span className="btn-icon">⬇</span>
                                <span className="btn-label">Download</span>
                            </button>
                            {file.can_delete && (
                                <button className="footer-btn delete-btn" onClick={handleDelete} title="Delete">
                                    <span className="btn-icon">🗑️</span>
                                    <span className="btn-label">Delete</span>
                                </button>
                            )}
                        </div>
                    </div>
                </div>

                {showDetails && (
                    <div className="preview-details-panel">
                        <div className="details-header">
                            <h3>Details</h3>
                            <button className="close-details-btn" onClick={() => setShowDetails(false)}>✕</button>
                        </div>
                        <div className="details-content">
                            <div className="detail-item">
                                <label>File Name</label>
                                <p>{file.name}</p>
                            </div>
                            <div className="detail-item">
                                <label>Type</label>
                                <p>{file.mime_type || 'Unknown'}</p>
                            </div>
                            <div className="detail-item">
                                <label>Size</label>
                                <p>{file.size_bytes ? (file.size_bytes / 1024).toFixed(2) + ' KB' : 'Unknown'}</p>
                            </div>
                            <div className="detail-item">
                                <label>Location</label>
                                <p>{file.path}</p>
                            </div>
                            <div className="detail-item">
                                <label>Created</label>
                                <p>{new Date(file.created_at).toLocaleString()}</p>
                            </div>
                            <div className="detail-item">
                                <label>Modified</label>
                                <p>{new Date(file.updated_at).toLocaleString()}</p>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
};

export default ImagePreviewModal;
