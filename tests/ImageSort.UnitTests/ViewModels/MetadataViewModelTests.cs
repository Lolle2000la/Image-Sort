using ImageSort.FileSystem;
using ImageSort.ViewModels.Metadata;
using Moq;
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

    private readonly Mock<IFileSystem> fileSystem = new();
    private readonly Mock<IMetadataExtractor> metadataExtractor = new();

    public MetadataViewModelTests()
    {
        metadataViewModel = new(metadataExtractor.Object, fileSystem.Object, new MetadataSectionViewModelFactory(new MetadataFieldViewModelFactory()));
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
        
        fileSystem.Setup(x => x.FileExists(thisFileExists)).Returns(true)
            .Verifiable("Should check whether or not the file exists");

        metadataExtractor.Setup(x => x.Extract(thisFileExists))
            .Returns(extractableMetadata)
            .Verifiable("Should extract metadata from image");

        

        // this should cause the extraction of metadata
        metadataViewModel.ImagePath = thisFileExists;

        Assert.Equal(MetadataResultType.Success, metadataViewModel.Metadata.Type);
        Assert.Equal(extractableMetadata, metadataViewModel.Metadata.Metadata);

        fileSystem.Verify(x => x.FileExists(thisFileExists));
        metadataExtractor.Verify(x => x.Extract(thisFileExists));
    }

    [Fact(DisplayName = "MetadataViewModel should not extract metadata from image when file does not exist")]
    public void DoesNotExtractMetadataWhenPathIsSetAndFileDoesNotExist()
    {
        // setup mocks and view model
        string thisFileDoesNotExist = "C:\\test2.jpg";
        
        fileSystem.Setup(x => x.FileExists(thisFileDoesNotExist)).Returns(false)
            .Verifiable("Should check whether or not the file exists");
        
        metadataExtractor.Setup(x => x.Extract(thisFileDoesNotExist))
            .Throws(new Exception("Should not extract metadata from image when file does not exist"))
            .Verifiable("Should not extract metadata from image when the file does not exist");

        metadataViewModel.ImagePath = thisFileDoesNotExist;

        Assert.Equal(MetadataResultType.FileDoesNotExist, metadataViewModel.Metadata.Type);
        Assert.Null(metadataViewModel.Metadata.Metadata);
        
        fileSystem.Verify(x => x.FileExists(thisFileDoesNotExist));
        metadataExtractor.Verify(x => x.Extract(thisFileDoesNotExist), Times.Never);
    }

    [Fact(DisplayName = "Correctly reports unhandled exceptions that occur when trying to extract metadata")]
    public void CorrectlyReportsIssuesWithTheExtractionOfMetadata()
    {
        // setup mocks and view model
        string thisFileHasInvalidMetadata = "C:\\test3.jpg";
        Exception invalidMetadata = new("Invalid metadata could not be loaded");
        
        fileSystem.Setup(x => x.FileExists(thisFileHasInvalidMetadata)).Returns(true)
            .Verifiable("Should check whether or not the file exists");

        metadataExtractor.Setup(x => x.Extract(thisFileHasInvalidMetadata))
            .Throws(invalidMetadata)
            .Verifiable("Should extract metadata from image when the file does exist");
        
        metadataViewModel.ImagePath = thisFileHasInvalidMetadata;

        Assert.Equal(MetadataResultType.UnexpectedError, metadataViewModel.Metadata.Type);
        Assert.Null(metadataViewModel.Metadata.Metadata);
        Assert.Equal(invalidMetadata, metadataViewModel.Metadata.Exception);

        fileSystem.Verify(x => x.FileExists(thisFileHasInvalidMetadata));
        metadataExtractor.Verify(x => x.Extract(thisFileHasInvalidMetadata));
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

        fileSystem.Setup(x => x.FileExists(thisFileHasMetadata)).Returns(true)
            .Verifiable("Should check whether or not the file exists");

        metadataExtractor.Setup(x => x.Extract(thisFileHasMetadata))
            .Returns(extractableMetadata)
            .Verifiable("Should extract metadata from image when the file does exist");

        metadataViewModel.ImagePath = thisFileHasMetadata;

        Assert.Equal(1, metadataViewModel.SectionViewModels.Count());
        Assert.Equal("test", metadataViewModel.SectionViewModels.First().Title);
        Assert.Equal(extractableMetadata["test"], metadataViewModel.SectionViewModels.First().Fields);

        fileSystem.Verify(x => x.FileExists(thisFileHasMetadata));
        metadataExtractor.Verify(x => x.Extract(thisFileHasMetadata));
    }
}
