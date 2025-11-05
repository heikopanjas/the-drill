use std::fmt;

// Re-export box types from individual modules
pub use crate::isobmff_chapter::ChapterBox;
pub use crate::{
    isobmff_data_reference::{DataReferenceBox, UrlEntryBox, UrnEntryBox},
    isobmff_edit_list::EditListBox,
    isobmff_file_type::FileTypeBox,
    isobmff_handler::HandlerBox,
    isobmff_media_header::MediaHeaderBox,
    isobmff_media_info_header::{NullMediaHeaderBox, SoundMediaHeaderBox, VideoMediaHeaderBox},
    isobmff_metadata_keys::{MetadataMeanBox, MetadataNameBox},
    isobmff_movie_header::MovieHeaderBox,
    isobmff_sample_table::{ChunkOffset64Box, ChunkOffsetBox, SampleDescriptionBox, SampleSizeBox, SampleToChunkBox, TimeToSampleBox},
    isobmff_track_header::TrackHeaderBox
};

/// Parsed ISOBMFF box content for various box types
#[derive(Debug, Clone)]
pub enum IsobmffContent
{
    FileType(FileTypeBox),
    MovieHeader(MovieHeaderBox),
    TrackHeader(TrackHeaderBox),
    MediaHeader(MediaHeaderBox),
    Handler(HandlerBox),
    VideoMediaHeader(VideoMediaHeaderBox),
    SoundMediaHeader(SoundMediaHeaderBox),
    NullMediaHeader(NullMediaHeaderBox),
    DataReference(DataReferenceBox),
    SampleDescription(SampleDescriptionBox),
    TimeToSample(TimeToSampleBox),
    SampleToChunk(SampleToChunkBox),
    SampleSize(SampleSizeBox),
    ChunkOffset(ChunkOffsetBox),
    ChunkOffset64(ChunkOffset64Box),
    EditList(EditListBox),
    UrlEntry(UrlEntryBox),
    UrnEntry(UrnEntryBox),
    Chapter(ChapterBox),
    MetadataMean(MetadataMeanBox),
    MetadataName(MetadataNameBox)
}

impl fmt::Display for IsobmffContent
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            | IsobmffContent::FileType(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::MovieHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::TrackHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::MediaHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::Handler(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::VideoMediaHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::SoundMediaHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::NullMediaHeader(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::DataReference(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::SampleDescription(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::TimeToSample(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::SampleToChunk(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::SampleSize(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::ChunkOffset(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::ChunkOffset64(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::EditList(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::UrlEntry(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::UrnEntry(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::Chapter(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::MetadataMean(box_data) => write!(f, "{}", box_data),
            | IsobmffContent::MetadataName(box_data) => write!(f, "{}", box_data)
        }
    }
}
