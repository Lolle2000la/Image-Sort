using System;

namespace ImageSort.FileSystem;

public class FileRestorationNotPossibleException : Exception
{
    public FileRestorationNotPossibleException()
    {
    }

    public FileRestorationNotPossibleException(string message) : base(message)
    {
    }

    public FileRestorationNotPossibleException(string message, Exception innerException) 
        : base(message, innerException)
    {
    }
}