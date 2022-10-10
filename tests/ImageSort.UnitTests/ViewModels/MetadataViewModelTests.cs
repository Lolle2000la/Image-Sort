using ImageSort.FileSystem;
using ImageSort.ViewModels;
using Moq;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Xunit;

namespace ImageSort.UnitTests.ViewModels
{
    public class MetadataViewModelTests
    {
        private readonly MetadataViewModel metadataViewModel;

        private readonly Mock<IFileSystem> fileSystem = new();
        private readonly Mock<IMetadataExtractor> metadataExtractor = new();

        public MetadataViewModelTests()
        {
            metadataViewModel = new(metadataExtractor.Object, fileSystem.Object);
        }


        [Fact(DisplayName = "MetadataViewModel should extract metadata from image")]
        public void ExtractsMetadataWhenPathIsSet()
        {
            // setup mocks and view model
            string mockPath = "C:\\test.jpg";
            var mockExtractedMetadata = new Dictionary<string, Dictionary<string, string>>(){
                { "test", new Dictionary<string, string>(){
                        { "test", "test" },
                    }
                }
            };
            
            fileSystem.Setup(x => x.FileExists(mockPath)).Returns(true)
                .Verifiable("Should check wheter or not the file exists");

            metadataExtractor.Setup(x => x.Extract(mockPath))
                .Returns(mockExtractedMetadata)
                .Verifiable("Should extract metadata from image");

            

            // this should cause the extraction of metadata
            metadataViewModel.ImagePath = mockPath;

            Assert.Equal(mockExtractedMetadata, metadataViewModel.Metadata);

            metadataExtractor.Verify(x => x.Extract(mockPath));
            fileSystem.Verify(x => x.FileExists(mockPath));
        }

        [Fact(DisplayName = "MetadataViewModel should not extract metadata from image when file does not exist")]
        public void DoesNotExtractMetadataWhenPathIsSetAndFileDoesNotExist()
        {
            // setup mocks and view model
            string thisFileExists = "C:\\test.jpg";
            string thisFileDoesNotExist = "C:\\test2.jpg";
            
            var mockExtractedMetadata = new Dictionary<string, Dictionary<string, string>>(){
                { "test", new Dictionary<string, string>(){
                        { "test", "test" },
                    }
                }
            };

            fileSystem.Setup(x => x.FileExists(thisFileExists)).Returns(true)
                .Verifiable("Should check wheter or not the file exists");
            fileSystem.Setup(x => x.FileExists(thisFileDoesNotExist)).Returns(false)
                .Verifiable("Should check wheter or not the file exists");

            metadataExtractor.Setup(x => x.Extract(thisFileExists))
                .Returns(mockExtractedMetadata)
                .Verifiable("Should extract metadata from image");
            metadataExtractor.Setup(x => x.Extract(thisFileDoesNotExist))
                .Throws(new Exception("Should not extract metadata from image when file does not exist"))
                .Verifiable("Should not extract metadata from image when file does not exist");

            metadataViewModel.ImagePath = thisFileExists;

            Assert.NotNull(metadataViewModel.Metadata);

            metadataViewModel.ImagePath = thisFileDoesNotExist;

            Assert.Null(metadataViewModel.Metadata);

            metadataExtractor.Verify(x => x.Extract(thisFileExists));
            metadataExtractor.Verify(x => x.Extract(thisFileDoesNotExist), Times.Never);
            fileSystem.Verify(x => x.FileExists(thisFileExists));
            fileSystem.Verify(x => x.FileExists(thisFileDoesNotExist));
        }
    }
}
