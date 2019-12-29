import React from 'react';
import { EditForm } from '../upload/UploadVideo'
import '../index.css';

import { ContentId } from '@joystream/types/media';
import { withMockTransport } from './withMockTransport';
import UploadVideoView from '../upload/UploadVideo.view';
import EntityId from '@joystream/types/versioned-store/EntityId';

export default { 
	title: 'Media | Upload video',
	decorators: [ withMockTransport ],
};

const contentId = ContentId.generate();

export const DefaultState = () =>
	<EditForm
		contentId={contentId} 
	/>;

export const MockEditFormView = () =>
	<UploadVideoView
		contentId={contentId}
		id={new EntityId(1)}
	/>;
