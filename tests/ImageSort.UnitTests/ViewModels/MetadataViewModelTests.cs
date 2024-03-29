﻿using ImageSort.FileSystem;
using ImageSort.ViewModels.Metadata;
using NSubstitute;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Xunit;

namespace ImageSort.UnitTests.ViewModels;

public class MetadataViewModelTests
{
    private readonly MetadataViewModel metadataViewModel;

    private readonly IFileSystem fileSystem = Substitute.For<IFileSystem>();
    private readonly IMetadataExtractor metadataExtractor = Substitute.For<IMetadataExtractor>();

    public MetadataViewModelTests()
    {
        metadataViewModel = new(metadataExtractor, fileSystem, new MetadataSectionViewModelFactory(new MetadataFieldViewModelFactory()));
    }

    [Fact(DisplayName = "MetadataViewModel should extract metadata from image")]
    public void ExtractsMetadataWhenPathIsSet()
    {
        // setup mocks and view model
        string thisFileExists = "C:\\test.jpg";
        var extractableMetadata = new Dictionary<string, Dictionary<string, string>>(){
            { "test", new Dictionary<string, string>(){
                    { "test", "test" },
                }
            }
        };

        fileSystem.FileExists(thisFileExists).Returns(true);

        metadataExtractor.Extract(thisFileExists).Returns(extractableMetadata);

        // this should cause the extraction of metadata
        metadataViewModel.ImagePath = thisFileExists;

        Assert.Equal(MetadataResultType.Success, metadataViewModel.Metadata.Type);
        Assert.Equal(extractableMetadata, metadataViewModel.Metadata.Metadata);

        fileSystem.Received().FileExists(thisFileExists);
        metadataExtractor.Received().Extract(thisFileExists);
    }

    [Fact(DisplayName = "MetadataViewModel should not extract metadata from image when file does not exist")]
    public void DoesNotExtractMetadataWhenPathIsSetAndFileDoesNotExist()
    {
        // setup mocks and view model
        string thisFileDoesNotExist = "C:\\test2.jpg";

        fileSystem.FileExists(thisFileDoesNotExist).Returns(false);

        metadataViewModel.ImagePath = thisFileDoesNotExist;

        Assert.Equal(MetadataResultType.FileDoesNotExist, metadataViewModel.Metadata.Type);
        Assert.Null(metadataViewModel.Metadata.Metadata);

        fileSystem.Received().FileExists(thisFileDoesNotExist);
        metadataExtractor.DidNotReceive().Extract(thisFileDoesNotExist);
    }

    [Fact(DisplayName = "Correctly reports unhandled exceptions that occur when trying to extract metadata")]
    public void CorrectlyReportsIssuesWithTheExtractionOfMetadata()
    {
        // setup mocks and view model
        string thisFileHasInvalidMetadata = "C:\\test3.jpg";
        Exception invalidMetadata = new("Invalid metadata could not be loaded");

        fileSystem.FileExists(thisFileHasInvalidMetadata).Returns(true);

        metadataExtractor.Extract(thisFileHasInvalidMetadata).Returns(x => throw invalidMetadata);

        metadataViewModel.ImagePath = thisFileHasInvalidMetadata;

        Assert.Equal(MetadataResultType.UnexpectedError, metadataViewModel.Metadata.Type);
        Assert.Null(metadataViewModel.Metadata.Metadata);
        Assert.Equal(invalidMetadata, metadataViewModel.Metadata.Exception);

        fileSystem.Received().FileExists(thisFileHasInvalidMetadata);
        metadataExtractor.Received().Extract(thisFileHasInvalidMetadata);
    }

    [Fact(DisplayName = "Correctly creates metadata sections from extracted metadata")]
    public void CorrectlyCreatesMetadataSectionsFromExtractedMetadata()
    {
        // setup mocks and view model
        string thisFileHasMetadata = "C:\\test4.jpg";
        var extractableMetadata = new Dictionary<string, Dictionary<string, string>>(){
            { "test", new Dictionary<string, string>(){
                    { "test", "test" },
                }
            }
        };

        fileSystem.FileExists(thisFileHasMetadata).Returns(true);

        metadataExtractor.Extract(thisFileHasMetadata).Returns(extractableMetadata);

        metadataViewModel.ImagePath = thisFileHasMetadata;

        Assert.Equal(1, metadataViewModel.SectionViewModels.Count());
        Assert.Equal("test", metadataViewModel.SectionViewModels.First().Title);
        Assert.Equal(extractableMetadata["test"], metadataViewModel.SectionViewModels.First().Fields);

        fileSystem.Received().FileExists(thisFileHasMetadata);
        metadataExtractor.Received().Extract(thisFileHasMetadata);
    }
}